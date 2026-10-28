using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Flowstate.Core;
namespace Flowstate.App;
public sealed partial class MainWindow
{
    DateOnly calendarDate=DateOnly.FromDateTime(DateTime.Today);
    string calendarView="Woche";
    void CalendarPage()
    {
        Header("Plan","Dein Kalender","Termine, Aufgaben und Pausen – mit Platz dazwischen.");
        var view=Choice("Ansicht",["Tag","Woche","Monat","Agenda"],calendarView);view.SelectionChanged+=(_,_)=>{calendarView=view.SelectedItem.ToString()!;Navigate(current);};
        var picker=new CalendarDatePicker { Date=new DateTimeOffset(calendarDate.ToDateTime(TimeOnly.MinValue)) };picker.DateChanged+=(_,_)=>{if(picker.Date is {} d){calendarDate=DateOnly.FromDateTime(d.Date);Navigate(current);}};
        void Shift(int direction){calendarDate=calendarView=="Monat"?calendarDate.AddMonths(direction):calendarDate.AddDays(direction*(calendarView=="Tag"?1:7));Navigate(current);}
        page.Children.Add(Row(Button("←",()=>{Shift(-1);return Task.CompletedTask;}),Button("Heute",()=>{calendarDate=DateOnly.FromDateTime(DateTime.Today);Navigate(current);return Task.CompletedTask;}),Button("→",()=>{Shift(1);return Task.CompletedTask;}),picker,view));
        page.Children.Add(Row(Button("+ Termin / Zeitblock",()=>BlockDialog(null),true),Go("AutoPlan öffnen","AutoPlan")));
        int days=calendarView switch{"Tag"=>1,"Woche"=>7,"Monat"=>DateTime.DaysInMonth(calendarDate.Year,calendarDate.Month),_=>14};
        var start=calendarView=="Monat"?new DateOnly(calendarDate.Year,calendarDate.Month,1):calendarDate;
        for(int i=0;i<days;i++)
        {
            var day=start.AddDays(i);var blocks=vm.Data.Blocks.Where(b=>DateOnly.FromDateTime(b.Start.ToLocalTime().Date)<=day && DateOnly.FromDateTime(b.End.ToLocalTime().AddTicks(-1).Date)>=day).OrderBy(b=>b.Start).ToList();
            var box=Stack(Label(day.ToString("ddd, dd. MMM",System.Globalization.CultureInfo.GetCultureInfo("de-CH")),18));
            foreach(var b in blocks)
            {
                var overlap=vm.Data.Blocks.Any(other=>other.Id!=b.Id && b.Overlaps(other));
                box.Children.Add(Button($"{b.Start.ToLocalTime():HH:mm}–{b.End.ToLocalTime():HH:mm}   {b.Title}   · {b.Kind}"+(b.Locked?" · Fixiert":"")+(overlap?" · Überschneidung":""),()=>BlockDialog(b)));
            }
            if(blocks.Count==0)box.Children.Add(Muted("Freier Raum"));page.Children.Add(Card(box));
        }
    }
    async Task BlockDialog(CalendarBlock? original)
    {
        var b=original==null?new CalendarBlock { Start=DateTimeOffset.Now.AddHours(1),End=DateTimeOffset.Now.AddHours(2),Kind=BlockKind.Event }:Clone(original);
        var title=Input("Titel",b.Title);var date=new CalendarDatePicker { Header="Datum",Date=b.Start };var start=new TimePicker { Header="Beginn",Time=b.Start.ToLocalTime().TimeOfDay,ClockIdentifier="24HourClock" };var end=new TimePicker { Header="Ende",Time=b.End.ToLocalTime().TimeOfDay,ClockIdentifier="24HourClock" };
        var tasks=vm.OpenTasks.ToList();var task=Choice("Aufgabe",new[]{"Freier Termin"}.Concat(tasks.Select(t=>t.Title)),tasks.FirstOrDefault(t=>t.Id==b.TaskId)?.Title);var locked=new CheckBox { Content="Fixiert – bei Neuplanung erhalten",IsChecked=b.Locked };
        var delete=new CheckBox { Content="Diesen Kalendereintrag löschen",Visibility=original==null?Visibility.Collapsed:Visibility.Visible };
        if(!await Edit("Zeitblock",Stack(title,date,Row(start,end),task,locked,delete)))return;
        if(delete.IsChecked==true){if(await Confirm("Eintrag löschen?",b.Title)){vm.Data.Blocks.Remove(original!);vm.Data.PlanUndo=null;await vm.SaveAsync();Navigate(current);}return;}
        if(date.Date==null)throw new ArgumentException("Bitte Datum auswählen.");
        var window=Scheduler.LocalWindow(DateOnly.FromDateTime(date.Date.Value.Date),TimeOnly.FromTimeSpan(start.Time),TimeOnly.FromTimeSpan(end.Time),TimeZoneInfo.Local);
        b.Start=window.Start;b.End=window.End;b.TaskId=task.SelectedIndex>0?tasks[task.SelectedIndex-1].Id:null;b.Title=string.IsNullOrWhiteSpace(title.Text)?tasks.FirstOrDefault(t=>t.Id==b.TaskId)?.Title??"Termin":title.Text.Trim();b.Kind=b.TaskId.HasValue?BlockKind.Manual:BlockKind.Event;b.Locked=locked.IsChecked==true;b.PlanId=null;
        if(vm.Data.Blocks.Any(x=>x.Id!=b.Id && b.Overlaps(x)) && !await Confirm("Zeitkonflikt", "Dieser Zeitblock überschneidet sich mit einem vorhandenen Eintrag. Trotzdem speichern?"))return;
        if(original!=null)vm.Data.Blocks.Remove(original);vm.Data.Blocks.Add(b);vm.Data.PlanUndo=null;await vm.SaveAsync();Navigate(current);
    }
    void AutoPlanPage()
    {
        Header("Plan → Focus","Weniger planen. Mehr Flow.","Ein realistischer Vorschlag, der deine Termine und Grenzen respektiert.");
        var mode=Choice("Planungsmodus",["Heute","Morgen","Nächste 7 Tage","Freies Zeitfenster","Quick Session"]);
        var date=new CalendarDatePicker { Header="Datum",Date=DateTimeOffset.Now };
        var start=new TimePicker { Header="Von",Time=TimeSpan.FromHours(vm.Data.Settings.WorkStart),ClockIdentifier="24HourClock" };var end=new TimePicker { Header="Bis",Time=TimeSpan.FromHours(Math.Min(23,vm.Data.Settings.WorkEnd)),ClockIdentifier="24HourClock" };
        var context=Input("Kontext (optional)");var energy=Choice("Verfügbare Energie",Enum.GetNames<Energy>(),"High");var quick=Number("Quick Session (Min.)",25);var pause=Number("Pause + Puffer (Min.)",vm.Data.Settings.ShortBreak+vm.Data.Settings.BufferMinutes,0,120);
        var preview=Stack();PlanResult? result=null;List<CalendarBlock>? baseBlocks=null;string? snapshot=null;
        page.Children.Add(Card(Stack(Row(mode,date),Row(start,end),Row(energy,quick,pause),context)));
        page.Children.Add(Button("Plan berechnen",()=>
        {
            var day=mode.SelectedIndex==1?DateOnly.FromDateTime(DateTime.Today.AddDays(1)):DateOnly.FromDateTime(date.Date?.Date??DateTime.Today);
            var windows=new List<TimeWindow>();var now=DateTimeOffset.Now;
            for(int i=0;i<(mode.SelectedIndex==2?7:1);i++)
            {
                var w=mode.SelectedIndex==4?new TimeWindow(now,now.AddMinutes(quick.Value)):Scheduler.LocalWindow(day.AddDays(i),TimeOnly.FromTimeSpan(start.Time),TimeOnly.FromTimeSpan(end.Time),TimeZoneInfo.Local);
                if(w.End>now)windows.Add(new(w.Start<now?now:w.Start,w.End));
            }
            var from=windows.Select(w=>w.Start).DefaultIfEmpty(now).Min();var to=windows.Select(w=>w.End).DefaultIfEmpty(now).Max();
            baseBlocks=vm.Data.Blocks.Where(b=>b.Locked || b.Kind is BlockKind.Manual or BlockKind.Event || b.Start<from || b.End>to).ToList();
            snapshot=System.Text.Json.JsonSerializer.Serialize(vm.Data);
            result=Scheduler.Plan(vm.Data.Tasks,baseBlocks,new(windows,context.Text.Trim(),Enum.Parse<Energy>(energy.SelectedItem.ToString()!), (int)pause.Value));preview.Children.Clear();
            preview.Children.Add(Label("Deine Planvorschau",23));
            foreach(var b in result.Blocks)preview.Children.Add(Card(Stack(Label($"{When(b.Start)}–{b.End.ToLocalTime():HH:mm} · {b.Title}",17),Muted(b.Reason))));
            foreach(var u in result.Unplanned)preview.Children.Add(Muted($"Nicht eingeplant: {vm.Data.Tasks.First(t=>t.Id==u.TaskId).Title} · {u.Reason}"));
            if(result.Blocks.Count==0)preview.Children.Add(Muted("Keine passenden Aufgaben oder freien Zeitfenster gefunden."));
            return Task.CompletedTask;
        },true));page.Children.Add(preview);
        page.Children.Add(Row(Button("Plan übernehmen",async()=>
        {
            if(result==null || baseBlocks==null || result.Blocks.Count==0) {Show("Berechne zuerst einen Plan mit Zeitblöcken.");return;}
            if(snapshot!=System.Text.Json.JsonSerializer.Serialize(vm.Data))throw new InvalidOperationException("Daten wurden geändert. Bitte den Plan neu berechnen.");
            vm.Data.PlanUndo=Clone(vm.Data.Blocks);vm.Data.Blocks=[..baseBlocks,..result.Blocks];await vm.SaveAsync();Show("Plan gespeichert. Manuelle und fixierte Termine bleiben erhalten.");Navigate("Kalender");
        },true),Button("Letzte Planung rückgängig",async()=>
        {
            if(vm.Data.PlanUndo==null){Show("Keine Planung zum Rückgängigmachen vorhanden.");return;}vm.Data.Blocks=vm.Data.PlanUndo;vm.Data.PlanUndo=null;await vm.SaveAsync();Navigate("Kalender");
        }),Go("Abbrechen","Kalender")));
    }
}
