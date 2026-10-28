using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Windowing;
using Flowstate.Core;
using Flowstate.Infrastructure;
using System.Diagnostics;
namespace Flowstate.App;
public sealed partial class MainWindow
{
    async Task StartFocus(Guid? taskId,int minutes,bool isBreak=false,bool stopwatch=false)
    {
        if(vm.Data.ActiveTimer!=null)
        {
            if(!await Confirm("Laufende Sitzung beenden?","Die bisherige Fokuszeit wird gespeichert."))return;
            await vm.FinishFocusAsync(false);
        }
        vm.Data.ActiveTimer=new() { TaskId=taskId,TargetSeconds=minutes*60,IsBreak=isBreak,Stopwatch=stopwatch };vm.Data.ActiveTimer.Start(DateTimeOffset.UtcNow);await vm.SaveAsync();Navigate("Focus Studio");
    }
    public async Task ToggleTimerAsync()
    {
        await Run(async()=> { var t=vm.Data.ActiveTimer;if(t==null){await StartFocus(null,vm.Data.Settings.FocusMinutes);return;}if(t.Running)t.Pause(DateTimeOffset.UtcNow);else t.Start(DateTimeOffset.UtcNow);await vm.SaveAsync();Navigate("Focus Studio"); });
    }
    async Task Tick()
    {
        var t=vm.Data.ActiveTimer;if(t==null)return;var now=DateTimeOffset.UtcNow;
        if(timerText!=null)timerText.Text=TimeSpan.FromSeconds(Math.Floor(t.Stopwatch?t.Elapsed(now):t.Remaining(now))).ToString(t.Elapsed(now)>=3600?@"hh\:mm\:ss":@"mm\:ss");
        if(timerProgress!=null)timerProgress.Value=Math.Min(100,t.Elapsed(now)/Math.Max(1,t.TargetSeconds)*100);
        if(t.Running && t.Finished(now))
        {
            var wasBreak=t.IsBreak;await vm.FinishFocusAsync(true);if(!wasBreak)completedFocusCount++;
            if(vm.Data.Settings.Notifications)native?.Notify(wasBreak?"Pause beendet":"Gut gemacht. Zeit für eine Pause.",wasBreak?"Deine nächste Fokussitzung wartet.":"Deine Fokuszeit wurde gespeichert.");
            Show(wasBreak?"Pause beendet. Bereit für die nächste Sitzung?":"Fokussitzung abgeschlossen. Gönn dir eine Pause.");
            if(current=="Focus Studio")Navigate(current);
        }
    }
    void FocusPage()
    {
        Header("Focus","Ein Moment. Eine Aufgabe.","Schaffe Raum für konzentriertes Arbeiten und bewusste Pausen.");
        var tasks=vm.OpenTasks.ToList();var task=Choice("Aufgabe",new[]{"Freier Fokus"}.Concat(tasks.Select(t=>t.Title)),tasks.FirstOrDefault(t=>t.Id==vm.Data.ActiveTimer?.TaskId)?.Title);
        var mode=Choice("Modus",["Pomodoro","Deep Work","Benutzerdefiniert","Stoppuhr"]);var duration=Number("Fokusdauer (Minuten)",vm.Data.Settings.FocusMinutes);
        mode.SelectionChanged+=(_,_)=>{duration.Value=mode.SelectedIndex==1?90:vm.Data.Settings.FocusMinutes;};
        var t=vm.Data.ActiveTimer;timerText=Label(t==null?TimeSpan.FromMinutes(vm.Data.Settings.FocusMinutes).ToString(@"mm\:ss"):TimeSpan.FromSeconds(t.Stopwatch?t.Elapsed(DateTimeOffset.UtcNow):t.Remaining(DateTimeOffset.UtcNow)).ToString(@"hh\:mm\:ss"),88);timerText.HorizontalAlignment=HorizontalAlignment.Center;timerText.FontFamily=new Microsoft.UI.Xaml.Media.FontFamily("Segoe UI Variable Display");
        timerProgress=new ProgressBar { Maximum=100,Value=t==null?0:100*t.Elapsed(DateTimeOffset.UtcNow)/Math.Max(1,t.TargetSeconds) };
        var title=t?.TaskId is {} id?vm.Data.Tasks.FirstOrDefault(x=>x.Id==id)?.Title:"Dein Fokus beginnt hier";
        var card=Stack(Muted(t?.IsBreak==true?"PAUSE":t?.Running==true?"IM FLOW":t==null?"BEREIT, WENN DU ES BIST":"PAUSIERT"),Label(title??"Freier Fokus",22),timerText,timerProgress);
        if(t==null)card.Children.Add(Button("Fokus starten",()=>StartFocus(task.SelectedIndex>0?tasks[task.SelectedIndex-1].Id:null,(int)duration.Value,false,mode.SelectedIndex==3),true));
        else card.Children.Add(Row(Button(t.Running?"Pausieren":"Fortsetzen",ToggleTimerAsync,true),Button("Beenden & speichern",async()=>{await vm.FinishFocusAsync(false);Navigate(current);}),Button("+ 5 Minuten",async()=>{t.TargetSeconds+=300;await vm.SaveAsync();}),Button("Zurücksetzen",async()=>{if(await Confirm("Timer zurücksetzen?","Die aktuelle Sitzung wird verworfen.")){vm.Data.ActiveTimer=null;await vm.SaveAsync();Navigate(current);}})));
        page.Children.Add(Card(card));page.Children.Add(Card(Stack(Row(task,mode,duration),Muted("Standby zählt zur Sitzungszeit. Pausen zählen nicht als Fokuszeit. Der Timer bleibt beim Seitenwechsel und im Infobereich aktiv."))));
        var breakMinutes=completedFocusCount>0 && completedFocusCount%vm.Data.Settings.SessionsBeforeLongBreak==0?vm.Data.Settings.LongBreak:vm.Data.Settings.ShortBreak;
        page.Children.Add(Row(Button($"Pause starten · {breakMinutes} Min.",()=>StartFocus(null,breakMinutes,true)),Go("Klangraum öffnen","Sound Studio"),Button("Windows-Fokuseinstellungen",()=>OpenTarget("ms-settings:quietmomentshome",true))));
        var compact=new ToggleSwitch { Header="Kompakter Timer im Vordergrund" };compact.Toggled+=(_,_)=> { if(compact.IsOn)AppWindow.SetPresenter(AppWindowPresenterKind.CompactOverlay);else AppWindow.SetPresenter(AppWindowPresenterKind.Default); };page.Children.Add(compact);
        page.Children.Add(Muted("Windows „Nicht stören“ lässt sich hier über die Systemeinstellungen aktivieren. Flowstate verändert diesen Systemzustand nicht automatisch."));
    }
    void SoundPage()
    {
        Header("Sound Studio","Dein persönlicher Klangraum.","Lokale Klänge, sanfte Übergänge und keine Unterbrechungen.");
        Slider Volume(string name,double value,Action<double> changed)
        {
            var s=new Slider { Header=name,Minimum=0,Maximum=100,Value=value*100,StepFrequency=1 };s.ValueChanged+=(_,_)=>changed(s.Value/100);return s;
        }
        audio.White=vm.Data.Settings.WhiteVolume;audio.Brown=vm.Data.Settings.BrownVolume;audio.Pink=vm.Data.Settings.PinkVolume;audio.Master=vm.Data.Settings.MasterVolume;
        page.Children.Add(Card(Stack(Label("Noise Mixer",22),Muted("Live erzeugt · White, Brown und Pink Noise · vollständig offline"),Volume("White Noise",audio.White,v=>audio.White=v),Volume("Brown Noise",audio.Brown,v=>audio.Brown=v),Volume("Pink Noise",audio.Pink,v=>audio.Pink=v),Volume("Gesamtlautstärke",audio.Master,v=>audio.Master=v))));
        page.Children.Add(Row(Button("Abspielen",()=>{audio.Play();return Task.CompletedTask;},true),Button("Pause / Stille",()=>{audio.Pause();return Task.CompletedTask;}),Button("Mix als Standard speichern",async()=>{vm.Data.Settings.WhiteVolume=audio.White;vm.Data.Settings.BrownVolume=audio.Brown;vm.Data.Settings.PinkVolume=audio.Pink;vm.Data.Settings.MasterVolume=audio.Master;await vm.SaveAsync();Show("Klangmischung gespeichert.");})));
        var path=Input("Eigene Audiodatei (WAV / MP3 / FLAC)");
        page.Children.Add(Card(Stack(Label("Deine Atmosphäre",22),Muted("Importiere eigene Aufnahmen für Regen, Wald, Café oder Musik. Es werden keine fremden Musikstücke mitgeliefert."),path,Button("Datei auswählen",async()=>{var selected=await PickFile([".wav",".mp3",".flac"]);if(selected!=null)path.Text=selected;}),Volume("Audiodatei",audio.FileVolume,v=>audio.FileVolume=v),Button("Datei in Schleife abspielen",()=>{audio.Load(path.Text);audio.Play();Show("Audiodatei geladen.");return Task.CompletedTask;}))));
    }
    void WorkspacesPage()
    {
        Header("Workspaces","Alles bereit für deinen Flow.","Öffne die Werkzeuge und Materialien für deine nächste Arbeitseinheit.");
        page.Children.Add(Button("+ Workspace erstellen",()=>WorkspaceDialog(null),true));
        foreach(var w in vm.Data.Workspaces)page.Children.Add(Card(Stack(Label(w.Name,22),Muted(w.Description),Muted(string.Join("\n",w.Targets)),Row(Button("Workspace starten",async()=>
        {
            if(!await Confirm("Workspace starten?",string.Join("\n",w.Targets)+$"\n\nFokusdauer: {w.FocusMinutes} Minuten"))return;
            foreach(var target in w.Targets)await OpenTarget(target);await StartFocus(null,w.FocusMinutes);
        },true),Button("Bearbeiten",()=>WorkspaceDialog(w)),Button("Löschen",async()=>{if(await Confirm("Workspace löschen?",w.Name)){vm.Data.Workspaces.Remove(w);await vm.SaveAsync();Navigate(current);}})))));
        if(vm.Data.Workspaces.Count==0)Empty("Ein Platz für jede Arbeitsweise","Erstelle zum Beispiel einen Workspace für Lernen, Schreiben oder Programmieren.");
    }
    async Task WorkspaceDialog(Workspace? original)
    {
        var w=original==null?new Workspace():Clone(original);var name=Input("Name",w.Name);var description=Input("Beschreibung",w.Description);var targets=Input("Programme, Ordner, Dateien oder HTTPS-URLs (eine pro Zeile)",string.Join('\n',w.Targets),true);var duration=Number("Fokusdauer",w.FocusMinutes);
        if(!await Edit("Workspace",Stack(name,description,targets,duration)))return;
        if(string.IsNullOrWhiteSpace(name.Text))throw new ArgumentException("Bitte einen Namen eingeben.");w.Name=name.Text.Trim();w.Description=description.Text;w.Targets=targets.Text.Split('\n',StringSplitOptions.RemoveEmptyEntries|StringSplitOptions.TrimEntries).ToList();foreach(var target in w.Targets)ValidateTarget(target);w.FocusMinutes=(int)duration.Value;
        if(original!=null)vm.Data.Workspaces.Remove(original);vm.Data.Workspaces.Add(w);await vm.SaveAsync();Navigate(current);
    }
    static void ValidateTarget(string target)
    {
        if(Uri.TryCreate(target,UriKind.Absolute,out var u) && u.Scheme is "https" or "http") {if(!string.IsNullOrEmpty(u.UserInfo))throw new ArgumentException("URLs mit Anmeldedaten sind nicht erlaubt.");return;}
        if(!Path.IsPathFullyQualified(target) || (!File.Exists(target)&&!Directory.Exists(target)))throw new ArgumentException("Bitte einen existierenden absoluten Dateipfad oder eine HTTP(S)-URL angeben.");
        if(File.Exists(target) && !new[]{".exe",".pdf",".txt",".md",".docx",".xlsx",".pptx",".png",".jpg"}.Contains(Path.GetExtension(target).ToLowerInvariant()))throw new ArgumentException("Dieser Dateityp wird aus Sicherheitsgründen nicht als Workspace-Aktion unterstützt.");
    }
    static Task OpenTarget(string target,bool settings=false)
    { if(!settings)ValidateTarget(target);else if(target!="ms-settings:quietmomentshome")throw new ArgumentException("Ungültige Einstellungsadresse.");Process.Start(new ProcessStartInfo(target){UseShellExecute=true});return Task.CompletedTask; }
    async Task<string?> PickFile(string[] extensions)
    {
        var picker=new Windows.Storage.Pickers.FileOpenPicker();WinRT.Interop.InitializeWithWindow.Initialize(picker,WinRT.Interop.WindowNative.GetWindowHandle(this));foreach(var e in extensions)picker.FileTypeFilter.Add(e);var file=await picker.PickSingleFileAsync();return file?.Path;
    }
}
