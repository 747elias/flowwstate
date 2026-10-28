using Microsoft.UI;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Windowing;
using Windows.Graphics;
using Flowstate.Core;
using Flowstate.Infrastructure;
using System.Globalization;
using System.Text.Json;
using System.Diagnostics;
namespace Flowstate.App;
public sealed partial class MainWindow : Window
{
    readonly MainViewModel vm;
    readonly NavigationView nav = new() { IsBackButtonVisible=NavigationViewBackButtonVisible.Collapsed, IsSettingsVisible=false, OpenPaneLength=236, PaneDisplayMode=NavigationViewPaneDisplayMode.Auto, CompactModeThresholdWidth=860, ExpandedModeThresholdWidth=1100, IsPaneToggleButtonVisible=true };
    readonly StackPanel page = new() { Spacing=22, Padding=new Thickness(32,22,32,40), MaxWidth=1500, HorizontalAlignment=HorizontalAlignment.Stretch };
    readonly Grid root = new();
    readonly InfoBar notice = new() { IsOpen=false, IsClosable=true, Margin=new Thickness(32,4,32,0) };
    readonly DispatcherTimer ticker = new() { Interval=TimeSpan.FromSeconds(1) };
    readonly AudioMixer audio = new();
    NativeIntegration? native;
    string current="Dashboard";
    bool ready, exiting, ticking;
    TextBlock? timerText;
    ProgressBar? timerProgress;
    int completedFocusCount;
    readonly string[] pages=["Dashboard","Aufgaben","Projekte","Kalender","AutoPlan","Focus Studio","Sound Studio","Workspaces","Analytics","Tagesrückblick","Einstellungen","Über Flowstate"];
    public MainWindow(MainViewModel model)
    {
        vm=model; Title="Flowstate"; AppWindow.Resize(new SizeInt32(1320,900));
        SystemBackdrop=new MicaBackdrop();
        var iconPath=Path.Combine(AppContext.BaseDirectory,"Assets","Flowstate.ico"); if(File.Exists(iconPath)) AppWindow.SetIcon(iconPath);
        root.RowDefinitions.Add(new RowDefinition { Height=GridLength.Auto }); root.RowDefinitions.Add(new RowDefinition { Height=new GridLength(1,GridUnitType.Star) });
        var titlebar = new Grid { Height=48, Padding=new Thickness(24,0,140,0) }; titlebar.ColumnDefinitions.Add(new ColumnDefinition());
        titlebar.Children.Add(new TextBlock { Text="◈   FLOWSTATE", FontSize=13, CharacterSpacing=160, VerticalAlignment=VerticalAlignment.Center });
        root.Children.Add(titlebar); ExtendsContentIntoTitleBar=true; SetTitleBar(titlebar);
        var brand=new StackPanel { Margin=new Thickness(20,20,16,26), Spacing=6 };
        brand.Children.Add(Label("Your time. Your focus.",18)); brand.Children.Add(Muted("Your flow.",15)); nav.PaneHeader=brand;
        string[] glyphs=["\uE80F","\uE73A","\uE8B7","\uE787","\uE945","\uE916","\uE767","\uE7F4","\uE9D9","\uE8A5","\uE713","\uE946"];
        for(int i=0;i<pages.Length;i++)
        {
            var item=new NavigationViewItem { Content=pages[i], Tag=pages[i], Icon=new FontIcon { Glyph=glyphs[i] } };
            if(i>=10) nav.FooterMenuItems.Add(item); else nav.MenuItems.Add(item);
        }
        var host=new Grid(); host.RowDefinitions.Add(new RowDefinition { Height=GridLength.Auto }); host.RowDefinitions.Add(new RowDefinition());
        host.Children.Add(notice); var scroll=new ScrollViewer { Content=page, HorizontalScrollBarVisibility=ScrollBarVisibility.Disabled }; Grid.SetRow(scroll,1); host.Children.Add(scroll); nav.Content=host;
        Grid.SetRow(nav,1); root.Children.Add(nav); Content=root;
        nav.SelectionChanged+=(_,e)=> { if(ready && e.SelectedItem is NavigationViewItem i) Navigate(i.Tag.ToString()!); };
        root.Loaded+=async (_,_)=> { if(ready)return; await Run(async()=> { await vm.LoadAsync(); ready=true; ApplyTheme(); native=new NativeIntegration(this,()=>Run(CaptureDialog), ()=>Navigate("Focus Studio")); native.Register(vm.Data.Settings.Hotkey); Navigate(vm.Data.Settings.StartPage); ticker.Start(); await CheckUpdates(false); }); };
        ticker.Tick+=async (_,_)=> { if(ticking || !ready)return; ticking=true; try { await Tick(); } catch(Exception e){ App.Log(e); Show(e.Message,true); } finally { ticking=false; } };
        AppWindow.Closing+=async (_,e)=>
        {
            if(exiting) return;
            e.Cancel=true;
            if(ready && vm.Data.Settings.MinimizeToTray) { AppWindow.Hide(); Show("Flowstate läuft im Infobereich weiter."); }
            else await ExitAsync();
        };
        Closed+=(_,_)=> { ticker.Stop(); audio.Dispose(); native?.Dispose(); };
    }
    public void ShowWindow() { AppWindow.Show(); Activate(); }
    public async Task ExitAsync() { await Run(async()=> { if(ready)await vm.SaveAsync(); exiting=true; Close(); }); }
    void ApplyTheme()
    {
        root.RequestedTheme=vm.Data.Settings.Theme switch { "Light"=>ElementTheme.Light, "System"=>ElementTheme.Default, _=>ElementTheme.Dark };
        Application.Current.Resources["FlowAccent"]=new SolidColorBrush(ParseColor(vm.Data.Settings.Accent));
    }
    static Windows.UI.Color ParseColor(string value) { try{return Windows.UI.Color.FromArgb(255,Convert.ToByte(value[1..3],16),Convert.ToByte(value[3..5],16),Convert.ToByte(value[5..7],16));}catch{return Windows.UI.Color.FromArgb(255,139,128,255);} }
    Brush Resource(string key) => (Brush)Application.Current.Resources[key];
    TextBlock Label(string text,double size=14) => new() { Text=text, FontSize=size, TextWrapping=TextWrapping.Wrap, VerticalAlignment=VerticalAlignment.Center };
    TextBlock Muted(string text,double size=13) { var t=Label(text,size); t.Foreground=Resource("MutedForeground"); return t; }
    StackPanel Stack(params UIElement[] children) { var p=new StackPanel { Spacing=12 }; foreach(var c in children)p.Children.Add(c); return p; }
    StackPanel Row(params UIElement[] children) { var p=new StackPanel { Orientation=Orientation.Horizontal, Spacing=12 }; foreach(var c in children)p.Children.Add(c); return p; }
    Border Card(UIElement child) => new() { Background=Resource("CardBackground"), BorderBrush=Resource("CardStroke"), BorderThickness=new Thickness(1), CornerRadius=new CornerRadius(14), Padding=new Thickness(22), Child=child };
    Button Button(string text, Func<Task> action, bool accent=false)
    {
        var b=new Button { Content=text, Padding=new Thickness(16,10,16,10) };
        if(accent)b.Style=(Style)Application.Current.Resources["AccentButtonStyle"];
        b.Click+=async (_,_)=> { b.IsEnabled=false; try{await Run(action);}finally{b.IsEnabled=true;} }; return b;
    }
    Button Go(string text,string destination) => Button(text,()=> { Navigate(destination); return Task.CompletedTask; });
    TextBox Input(string header,string text="", bool multiline=false) => new() { Header=header, Text=text, AcceptsReturn=multiline, TextWrapping=TextWrapping.Wrap, MinHeight=multiline?80:0 };
    NumberBox Number(string label,double value,double min=1,double max=1440) => new() { Header=label, Value=value, Minimum=min, Maximum=max, SpinButtonPlacementMode=NumberBoxSpinButtonPlacementMode.Compact, Width=190 };
    ComboBox Choice(string header,IEnumerable<string> options,string? selected=null)
    {
        var c=new ComboBox { Header=header, MinWidth=170, HorizontalAlignment=HorizontalAlignment.Stretch }; foreach(var o in options)c.Items.Add(o); c.SelectedItem=selected; if(c.SelectedIndex<0)c.SelectedIndex=0; return c;
    }
    void Header(string eyebrow,string title,string subtitle) { var e=Label(eyebrow.ToUpperInvariant(),11); e.CharacterSpacing=180; e.Foreground=Resource("FlowAccent"); page.Children.Add(Stack(e,Label(title,34),Muted(subtitle,15))); }
    void Empty(string title,string message) => page.Children.Add(Card(Stack(Label(title,20),Muted(message))));
    void Show(string text,bool error=false) { notice.Title=error?"Das hat nicht geklappt":"Flowstate"; notice.Message=text; notice.Severity=error?InfoBarSeverity.Error:InfoBarSeverity.Informational; notice.IsOpen=true; }
    async Task Run(Func<Task> action) { try{await action();}catch(Exception e){App.Log(e);Show(e.Message,true);} }
    async Task<bool> Confirm(string title,string description)
    { var d=new ContentDialog { XamlRoot=root.XamlRoot, Title=title, Content=description, PrimaryButtonText="Bestätigen", CloseButtonText="Abbrechen", DefaultButton=ContentDialogButton.Close, RequestedTheme=root.ActualTheme };return await d.ShowAsync()==ContentDialogResult.Primary; }
    async Task<bool> Edit(string title,StackPanel fields,string primary="Speichern")
    { var d=new ContentDialog { XamlRoot=root.XamlRoot, Title=title, Content=new ScrollViewer { Content=fields, MaxHeight=580 }, PrimaryButtonText=primary, CloseButtonText="Abbrechen", DefaultButton=ContentDialogButton.Primary, RequestedTheme=root.ActualTheme };return await d.ShowAsync()==ContentDialogResult.Primary; }
    void Navigate(string destination)
    {
        current=pages.Contains(destination)?destination:"Dashboard"; page.Children.Clear();timerText=null;timerProgress=null;
        var item=nav.MenuItems.Concat(nav.FooterMenuItems).OfType<NavigationViewItem>().First(i=>i.Tag.ToString()==current);
        if(nav.SelectedItem!=item){nav.SelectedItem=item;return;}
        switch(current)
        {
            case "Dashboard": Dashboard();break; case "Aufgaben": TasksPage();break; case "Projekte": ProjectsPage();break; case "Kalender": CalendarPage();break;
            case "AutoPlan": AutoPlanPage();break;case "Focus Studio": FocusPage();break;case "Sound Studio": SoundPage();break;case "Workspaces": WorkspacesPage();break;
            case "Analytics": AnalyticsPage();break;case "Tagesrückblick": ReviewPage();break;case "Einstellungen": SettingsPage();break;default: AboutPage();break;
        }
    }
    static string When(DateTimeOffset value)=>value.ToLocalTime().ToString("ddd, dd. MMM · HH:mm",CultureInfo.GetCultureInfo("de-CH"));
    static T Clone<T>(T obj)=>JsonSerializer.Deserialize<T>(JsonSerializer.Serialize(obj,Store.Json),Store.Json)!;
}
