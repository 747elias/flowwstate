using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Flowstate.Core;
namespace Flowstate.App;

public sealed partial class MainWindow
{
    DateOnly calendarDate = DateOnly.FromDateTime(DateTime.Today);
    string calendarView = "Woche";
    void CalendarPage()
    {
        Header("Plan", "Dein Kalender", "Termine, Aufgaben und Pausen – mit Platz dazwischen.");
        var view = Choice("Ansicht", ["Tag", "Woche", "Monat", "Agenda"], calendarView); view.SelectionChanged += (_, _) => { calendarView = view.SelectedItem.ToString()!; Navigate(current); };
        var picker = new CalendarDatePicker { Date = new DateTimeOffset(calendarDate.ToDateTime(TimeOnly.MinValue)) }; picker.DateChanged += (_, _) => { if (picker.Date is { } d) { calendarDate = DateOnly.FromDateTime(d.Date); Navigate(current); } };
        void Shift(int direction) { calendarDate = calendarView == "Monat" ? calendarDate.AddMonths(direction) : calendarDate.AddDays(direction * (calendarView == "Tag" ? 1 : 7)); Navigate(current); }
        page.Children.Add(Row(Button("←", () => { Shift(-1); return Task.CompletedTask; }), Button("Heute", () => { calendarDate = DateOnly.FromDateTime(DateTime.Today); Navigate(current); return Task.CompletedTask; }), Button("→", () => { Shift(1); return Task.CompletedTask; }), picker, view));
        page.Children.Add(Row(Button("+ Termin / Zeitblock", () => BlockDialog(null), true), Go("AutoPlan öffnen", "AutoPlan")));
        int days = calendarView switch { "Tag" => 1, "Woche" => 7, "Monat" => DateTime.DaysInMonth(calendarDate.Year, calendarDate.Month), _ => 14 };
        var start = calendarView == "Monat" ? new DateOnly(calendarDate.Year, calendarDate.Month, 1) : calendarDate;
        if (calendarView is "Tag" or "Woche") { CalendarGrid(start, days); return; }
        if (calendarView == "Monat") { MonthGrid(start, days); return; }
        for (int i = 0; i < days; i++)
        {
            var day = start.AddDays(i); var blocks = vm.Data.Blocks.Where(b => DateOnly.FromDateTime(b.Start.ToLocalTime().Date) <= day && DateOnly.FromDateTime(b.End.ToLocalTime().AddTicks(-1).Date) >= day).OrderBy(b => b.Start).ToList();
            var box = Stack(Label(day.ToString("ddd, dd. MMM", System.Globalization.CultureInfo.GetCultureInfo("de-CH")), 18));
            foreach (var b in blocks)
            {
                var overlap = vm.Data.Blocks.Any(other => other.Id != b.Id && b.Overlaps(other));
                box.Children.Add(Button($"{b.Start.ToLocalTime():HH:mm}–{b.End.ToLocalTime():HH:mm}   {b.Title}   · {b.Kind}" + (b.Locked ? " · Fixiert" : "") + (overlap ? " · Überschneidung" : ""), () => BlockDialog(b)));
            }
            if (blocks.Count == 0) box.Children.Add(Muted("Freier Raum")); page.Children.Add(Card(box));
        }
    }
    void CalendarGrid(DateOnly start, int days)
    {
        var tasks = Row(); foreach (var task in vm.OpenTasks.Take(10))
        {
            var chip = new Border { Child = Label(task.Title), Padding = new Thickness(10), CornerRadius = new CornerRadius(8), Background = Resource("CardBackground"), CanDrag = true };
            chip.DragStarting += (_, e) => { e.Data.SetText("task:" + task.Id); e.Data.RequestedOperation = Windows.ApplicationModel.DataTransfer.DataPackageOperation.Copy; }; tasks.Children.Add(chip);
        }
        page.Children.Add(Muted("Ziehe eine Aufgabe in ein Zeitfenster. Vor dem Speichern kannst du den Block bearbeiten.")); page.Children.Add(new ScrollViewer { Content = tasks, HorizontalScrollBarVisibility = ScrollBarVisibility.Auto, VerticalScrollBarVisibility = ScrollBarVisibility.Disabled });
        var grid = new Grid { ColumnSpacing = 8, MinWidth = days == 7 ? 850 : 400 }; grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(54) }); for (int i = 0; i < days; i++) grid.ColumnDefinitions.Add(new ColumnDefinition());
        var hours = new Microsoft.UI.Xaml.Controls.Canvas { Height = 24 * 64 + 40 }; for (int h = 0; h < 24; h++) { var text = Muted($"{h:00}:00", 11); Microsoft.UI.Xaml.Controls.Canvas.SetTop(text, h * 64 + 40); hours.Children.Add(text); }
        grid.Children.Add(hours);
        for (int i = 0; i < days; i++)
        {
            var day = start.AddDays(i); var column = Stack(Label(day.ToString("ddd dd.MM", System.Globalization.CultureInfo.GetCultureInfo("de-CH")), 14));
            var canvas = new Microsoft.UI.Xaml.Controls.Canvas { Height = 24 * 64, Background = Resource("CardBackground"), AllowDrop = true, HorizontalAlignment = HorizontalAlignment.Stretch };
            for (int h = 0; h < 24; h++) { var line = new Border { Height = 1, Width = 1000, Background = Resource("CardStroke") }; Microsoft.UI.Xaml.Controls.Canvas.SetTop(line, h * 64); canvas.Children.Add(line); }
            void Clip() { canvas.Clip = new Microsoft.UI.Xaml.Media.RectangleGeometry { Rect = new Windows.Foundation.Rect(0, 0, canvas.ActualWidth, canvas.Height) }; }
            canvas.SizeChanged += (_, _) => Clip();
            canvas.DoubleTapped += async (_, e) => await Run(async () => { int mins = Math.Clamp((int)(e.GetPosition(canvas).Y / 64 * 60) / 15 * 15, 0, 23 * 60); var w = Scheduler.LocalWindow(day, TimeOnly.FromTimeSpan(TimeSpan.FromMinutes(mins)), TimeOnly.FromTimeSpan(TimeSpan.FromMinutes(mins + 30)), TimeZoneInfo.Local); await BlockDialog(new CalendarBlock { Start = w.Start, End = w.End, Kind = BlockKind.Event }); });
            canvas.DragOver += (_, e) => e.AcceptedOperation = Windows.ApplicationModel.DataTransfer.DataPackageOperation.Copy;
            canvas.Drop += async (_, e) => await Run(async () =>
            {
                if (!e.DataView.Contains(Windows.ApplicationModel.DataTransfer.StandardDataFormats.Text)) return; var text = await e.DataView.GetTextAsync(); var parts = text.Split(':'); if (parts.Length != 2 || !Guid.TryParse(parts[1], out var id)) return;
                var task = vm.Data.Tasks.FirstOrDefault(t => t.Id == id); var existing = vm.Data.Blocks.FirstOrDefault(b => b.Id == id); if (parts[0] == "task" && task == null || parts[0] == "block" && existing == null) return;
                int mins = Math.Clamp((int)(e.GetPosition(canvas).Y / 64 * 60) / 15 * 15, 0, 23 * 60); var duration = task?.Minutes ?? (int)(existing!.End - existing.Start).TotalMinutes; duration = Math.Min(duration, 1439 - mins);
                var w = Scheduler.LocalWindow(day, TimeOnly.FromTimeSpan(TimeSpan.FromMinutes(mins)), TimeOnly.FromTimeSpan(TimeSpan.FromMinutes(mins + duration)), TimeZoneInfo.Local);
                var block = existing == null ? new CalendarBlock { TaskId = task!.Id, Title = task.Title, Kind = BlockKind.Manual } : Clone(existing); block.Start = w.Start; block.End = w.End; await BlockDialog(block);
            });
            foreach (var b in vm.Data.Blocks.Where(b => DateOnly.FromDateTime(b.Start.ToLocalTime().Date) <= day && DateOnly.FromDateTime(b.End.ToLocalTime().AddTicks(-1).Date) >= day))
            {
                var startMinutes = b.Start.ToLocalTime().Date < day.ToDateTime(TimeOnly.MinValue) ? 0 : b.Start.ToLocalTime().TimeOfDay.TotalMinutes;
                var endMinutes = b.End.ToLocalTime().Date > day.ToDateTime(TimeOnly.MinValue) ? 1440 : b.End.ToLocalTime().TimeOfDay.TotalMinutes;
                var button = Button($"{b.Start.ToLocalTime():HH:mm} {b.Title}", () => BlockDialog(b)); button.Height = Math.Max(28, (endMinutes - startMinutes) / 60 * 64 - 3); button.Padding = new Thickness(6); button.CanDrag = !b.Locked; button.Background = Resource(b.Kind == BlockKind.Break ? "CardStroke" : "FlowAccent");
                button.DragStarting += (_, e) => { e.Data.SetText("block:" + b.Id); e.Data.RequestedOperation = Windows.ApplicationModel.DataTransfer.DataPackageOperation.Move; };
                canvas.SizeChanged += (_, _) => button.Width = Math.Max(30, canvas.ActualWidth - 4); Microsoft.UI.Xaml.Controls.Canvas.SetTop(button, startMinutes / 60 * 64); canvas.Children.Add(button);
            }
            column.Children.Add(canvas); Grid.SetColumn(column, i + 1); grid.Children.Add(column);
        }
        page.Children.Add(new ScrollViewer { Content = grid, HorizontalScrollBarVisibility = ScrollBarVisibility.Auto, VerticalScrollBarVisibility = ScrollBarVisibility.Disabled });
    }
    void MonthGrid(DateOnly first, int days)
    {
        var grid = new Grid { ColumnSpacing = 8, RowSpacing = 8, MinWidth = 800 }; for (int i = 0; i < 7; i++) grid.ColumnDefinitions.Add(new ColumnDefinition()); for (int i = 0; i < 7; i++) grid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        var names = new[] { "Mo", "Di", "Mi", "Do", "Fr", "Sa", "So" }; for (int i = 0; i < 7; i++) { var label = Muted(names[i]); Grid.SetColumn(label, i); grid.Children.Add(label); }
        var offset = ((int)first.DayOfWeek + 6) % 7;
        for (int d = 0; d < days; d++)
        {
            var day = first.AddDays(d); var box = Stack(Label(day.Day.ToString(), 20)); foreach (var b in vm.Data.Blocks.Where(b => DateOnly.FromDateTime(b.Start.ToLocalTime().Date) == day).OrderBy(b => b.Start)) box.Children.Add(Button($"{b.Start.ToLocalTime():HH:mm} {b.Title}", () => BlockDialog(b)));
            var card = Card(box); card.MinHeight = 120; card.Padding = new Thickness(10); Grid.SetRow(card, (d + offset) / 7 + 1); Grid.SetColumn(card, (d + offset) % 7); grid.Children.Add(card);
        }
        page.Children.Add(new ScrollViewer { Content = grid, HorizontalScrollBarVisibility = ScrollBarVisibility.Auto, VerticalScrollBarVisibility = ScrollBarVisibility.Disabled });
    }
    async Task BlockDialog(CalendarBlock? original)
    {
        var b = original == null ? new CalendarBlock { Start = DateTimeOffset.Now.AddHours(1), End = DateTimeOffset.Now.AddHours(2), Kind = BlockKind.Event } : Clone(original);
        var title = Input("Titel", b.Title); var date = new CalendarDatePicker { Header = "Datum", Date = b.Start }; var start = new TimePicker { Header = "Beginn", Time = b.Start.ToLocalTime().TimeOfDay, ClockIdentifier = "24HourClock" }; var end = new TimePicker { Header = "Ende", Time = b.End.ToLocalTime().TimeOfDay, ClockIdentifier = "24HourClock" };
        var tasks = vm.OpenTasks.ToList(); var task = Choice("Aufgabe", new[] { "Freier Termin" }.Concat(tasks.Select(t => t.Title)), tasks.FirstOrDefault(t => t.Id == b.TaskId)?.Title); var locked = new CheckBox { Content = "Fixiert – bei Neuplanung erhalten", IsChecked = b.Locked };
        var delete = new CheckBox { Content = "Diesen Kalendereintrag löschen", Visibility = original == null ? Visibility.Collapsed : Visibility.Visible };
        if (!await Edit("Zeitblock", Stack(title, date, Row(start, end), task, locked, delete))) return;
        if (delete.IsChecked == true) { if (await Confirm("Eintrag löschen?", b.Title)) { vm.Data.Blocks.RemoveAll(x => x.Id == b.Id); vm.Data.PlanUndo = null; await vm.SaveAsync(); Navigate(current); } return; }
        if (date.Date == null) throw new ArgumentException("Bitte Datum auswählen.");
        var window = Scheduler.LocalWindow(DateOnly.FromDateTime(date.Date.Value.Date), TimeOnly.FromTimeSpan(start.Time), TimeOnly.FromTimeSpan(end.Time), TimeZoneInfo.Local);
        b.Start = window.Start; b.End = window.End; b.TaskId = task.SelectedIndex > 0 ? tasks[task.SelectedIndex - 1].Id : null; b.Title = string.IsNullOrWhiteSpace(title.Text) ? tasks.FirstOrDefault(t => t.Id == b.TaskId)?.Title ?? "Termin" : title.Text.Trim(); b.Kind = b.TaskId.HasValue ? BlockKind.Manual : BlockKind.Event; b.Locked = locked.IsChecked == true; b.PlanId = null;
        if (vm.Data.Blocks.Any(x => x.Id != b.Id && b.Overlaps(x)) && !await Confirm("Zeitkonflikt", "Dieser Zeitblock überschneidet sich mit einem vorhandenen Eintrag. Trotzdem speichern?")) return;
        if (original != null) vm.Data.Blocks.RemoveAll(x => x.Id == b.Id); vm.Data.Blocks.Add(b); vm.Data.PlanUndo = null; await vm.SaveAsync(); Navigate(current);
    }
    void AutoPlanPage()
    {
        Header("Plan → Focus", "Weniger planen. Mehr Flow.", "Ein realistischer Vorschlag, der deine Termine und Grenzen respektiert.");
        var mode = Choice("Planungsmodus", ["Heute", "Morgen", "Nächste 7 Tage", "Freies Zeitfenster", "Quick Session"]);
        var date = new CalendarDatePicker { Header = "Datum", Date = DateTimeOffset.Now };
        var start = new TimePicker { Header = "Von", Time = TimeSpan.FromHours(vm.Data.Settings.WorkStart), ClockIdentifier = "24HourClock" }; var end = new TimePicker { Header = "Bis", Time = TimeSpan.FromHours(Math.Min(23, vm.Data.Settings.WorkEnd)), ClockIdentifier = "24HourClock" };
        var context = Input("Kontext (optional)"); var energy = Choice("Verfügbare Energie", Enum.GetNames<Energy>(), "High"); var quick = Number("Quick Session (Min.)", 25); var pause = Number("Pause + Puffer (Min.)", vm.Data.Settings.ShortBreak + vm.Data.Settings.BufferMinutes, 0, 120);
        var preview = Stack(); PlanResult? result = null; List<CalendarBlock>? baseBlocks = null; string? snapshot = null;
        page.Children.Add(Card(Stack(Row(mode, date), Row(start, end), Row(energy, quick, pause), context)));
        page.Children.Add(Button("Plan berechnen", () =>
        {
            var day = mode.SelectedIndex == 1 ? DateOnly.FromDateTime(DateTime.Today.AddDays(1)) : DateOnly.FromDateTime(date.Date?.Date ?? DateTime.Today);
            var windows = new List<TimeWindow>(); var now = DateTimeOffset.Now;
            for (int i = 0; i < (mode.SelectedIndex == 2 ? 7 : 1); i++)
            {
                var w = mode.SelectedIndex == 4 ? new TimeWindow(now, now.AddMinutes(quick.Value)) : Scheduler.LocalWindow(day.AddDays(i), TimeOnly.FromTimeSpan(start.Time), TimeOnly.FromTimeSpan(end.Time), TimeZoneInfo.Local);
                if (w.End > now) windows.Add(new(w.Start < now ? now : w.Start, w.End));
            }
            var from = windows.Select(w => w.Start).DefaultIfEmpty(now).Min(); var to = windows.Select(w => w.End).DefaultIfEmpty(now).Max();
            baseBlocks = vm.Data.Blocks.Where(b => b.Locked || b.Kind is BlockKind.Manual or BlockKind.Event || b.Start < from || b.End > to).ToList();
            snapshot = System.Text.Json.JsonSerializer.Serialize(vm.Data);
            result = Scheduler.Plan(vm.Data.Tasks, baseBlocks, new(windows, context.Text.Trim(), Enum.Parse<Energy>(energy.SelectedItem.ToString()!), (int)pause.Value)); preview.Children.Clear();
            preview.Children.Add(Label("Deine Planvorschau", 23));
            foreach (var b in result.Blocks) preview.Children.Add(Card(Stack(Label($"{When(b.Start)}–{b.End.ToLocalTime():HH:mm} · {b.Title}", 17), Muted(b.Reason))));
            foreach (var u in result.Unplanned) preview.Children.Add(Muted($"Nicht eingeplant: {vm.Data.Tasks.First(t => t.Id == u.TaskId).Title} · {u.Reason}"));
            if (result.Blocks.Count == 0) preview.Children.Add(Muted("Keine passenden Aufgaben oder freien Zeitfenster gefunden."));
            return Task.CompletedTask;
        }, true)); page.Children.Add(preview);
        page.Children.Add(Row(Button("Plan übernehmen", async () =>
        {
            if (result == null || baseBlocks == null || result.Blocks.Count == 0) { Show("Berechne zuerst einen Plan mit Zeitblöcken."); return; }
            if (snapshot != System.Text.Json.JsonSerializer.Serialize(vm.Data)) throw new InvalidOperationException("Daten wurden geändert. Bitte den Plan neu berechnen.");
            vm.Data.PlanUndo = Clone(vm.Data.Blocks); vm.Data.Blocks = [.. baseBlocks, .. result.Blocks]; await vm.SaveAsync(); Show("Plan gespeichert. Manuelle und fixierte Termine bleiben erhalten."); Navigate("Kalender");
        }, true), Button("Letzte Planung rückgängig", async () =>
        {
            if (vm.Data.PlanUndo == null) { Show("Keine Planung zum Rückgängigmachen vorhanden."); return; }
            vm.Data.Blocks = vm.Data.PlanUndo; vm.Data.PlanUndo = null; await vm.SaveAsync(); Navigate("Kalender");
        }), Go("Abbrechen", "Kalender")));
    }
}
