using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Flowstate.Core;
using Flowstate.Infrastructure;
using System.Diagnostics;
using System.Reflection;
namespace Flowstate.App;

public sealed partial class MainWindow
{
    StackPanel WeekChart(int days)
    {
        var chart = Stack(); var since = DateTime.Today.AddDays(1 - days); var totals = Enumerable.Range(0, days).Select(i => { var day = since.AddDays(i); return (day, minutes: vm.Data.Sessions.Where(s => s.Start.ToLocalTime().Date == day).Sum(s => s.Minutes)); }).ToArray(); var max = Math.Max(60, totals.Max(t => t.minutes));
        foreach (var t in totals)
        {
            var row = new Grid { ColumnSpacing = 14 }; row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(90) }); row.ColumnDefinitions.Add(new ColumnDefinition()); row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(80) });
            row.Children.Add(Muted(t.day.ToString("ddd dd.MM"))); var bar = new ProgressBar { Value = t.minutes, Maximum = max, VerticalAlignment = VerticalAlignment.Center }; Grid.SetColumn(bar, 1); row.Children.Add(bar); var value = Muted($"{t.minutes:0} Min."); Grid.SetColumn(value, 2); row.Children.Add(value); chart.Children.Add(row);
        }
        return chart;
    }
    void AnalyticsPage()
    {
        Header("Review", "Dein Fortschritt, ohne Druck.", "Echte Zeiten aus deinen Fokussitzungen. Pausen gehören dazu.");
        if (!vm.Data.Settings.Analytics) { Empty("Auswertungen sind ausgeblendet", "Du kannst sie in den Einstellungen wieder anzeigen."); return; }
        var range = Choice("Zeitraum", ["Heute", "Letzte 7 Tage", "Letzte 30 Tage"]); range.SelectedIndex = 1; var content = Stack();
        void Render()
        {
            content.Children.Clear(); int days = range.SelectedIndex switch { 0 => 1, 2 => 30, _ => 7 }; var since = DateTime.Today.AddDays(1 - days); var sessions = vm.Data.Sessions.Where(s => s.Start.ToLocalTime().Date >= since).ToList();
            content.Children.Add(Card(Stack(Label($"{sessions.Sum(s => s.Minutes):0} Minuten im Fokus", 27), Muted($"{sessions.Count} Sitzungen · Ø {(sessions.Count == 0 ? 0 : sessions.Average(s => s.Minutes)):0} Minuten · {vm.Data.Tasks.Count(t => t.CompletedAt?.ToLocalTime().Date >= since)} Aufgaben abgeschlossen"), WeekChart(days))));
            foreach (var p in vm.Data.Projects)
            {
                var tasks = vm.Data.Tasks.Where(t => t.ProjectId == p.Id).ToList(); var ids = tasks.Select(t => t.Id).ToHashSet(); var minutes = sessions.Where(s => s.TaskId.HasValue && ids.Contains(s.TaskId.Value)).Sum(s => s.Minutes);
                content.Children.Add(Card(Stack(Label(p.Name, 20), Muted($"{minutes:0} Min. Fokus im Zeitraum · {tasks.Sum(t => t.Minutes)} Min. geschätzt (gesamt) · {tasks.Sum(t => t.ActualMinutes):0} Min. tatsächlich (gesamt)"))));
            }
        }
        range.SelectionChanged += (_, _) => Render(); page.Children.Add(range); page.Children.Add(content); Render();
    }
    void ReviewPage()
    {
        Header("Daily Review", "Ein guter Moment zum Innehalten.", "Was ist gelungen? Was kann bis morgen warten?");
        var today = DateOnly.FromDateTime(DateTime.Today); var done = vm.Data.Tasks.Where(t => t.CompletedAt?.ToLocalTime().Date == DateTime.Today).ToList(); var minutes = vm.Data.Sessions.Where(s => s.Start.ToLocalTime().Date == DateTime.Today).Sum(s => s.Minutes); var planned = vm.Data.Blocks.Where(b => b.Start.ToLocalTime().Date == DateTime.Today && b.TaskId != null).Sum(b => (b.End - b.Start).TotalMinutes);
        page.Children.Add(Card(Stack(Label($"{done.Count} Aufgaben abgeschlossen", 25), Muted($"{minutes:0} Minuten Fokus · {planned:0} Minuten eingeplant"), Muted(string.Join("\n", done.Select(t => "✓ " + t.Title))))));
        var review = vm.Data.Reviews.FirstOrDefault(r => r.Date == today); var notes = Input("Deine Gedanken", review?.Notes ?? "", true); page.Children.Add(Card(Stack(notes, Button("Rückblick speichern", async () => { if (review == null) { review = new DailyReview { Date = today }; vm.Data.Reviews.Add(review); } review.Notes = notes.Text; await vm.SaveAsync(); Show("Tagesrückblick gespeichert."); }, true))));
        page.Children.Add(Row(Go("Morgen planen", "AutoPlan"), Go("Kalender öffnen", "Kalender")));
    }
    void SettingsPage()
    {
        Header("Settings", "So, wie du arbeitest.", "Deine Daten bleiben auf diesem Computer. Kein Konto erforderlich.");
        var s = vm.Data.Settings; var theme = Choice("Darstellung", ["Dark", "Light", "System"], s.Theme); var accent = Choice("Akzentfarbe", ["#8B80FF", "#4ACBA5", "#58B8E8", "#E9AA65"], s.Accent); var startPage = Choice("Startseite", pages, s.StartPage);
        var tray = new ToggleSwitch { Header = "Beim Schließen im Infobereich weiterlaufen", IsOn = s.MinimizeToTray }; var notification = new ToggleSwitch { Header = "Benachrichtigungen bei Sitzungsende", IsOn = s.Notifications }; var analytics = new ToggleSwitch { Header = "Auswertungen anzeigen", IsOn = s.Analytics }; var updates = new ToggleSwitch { Header = "Täglich nach stabilen Updates suchen", IsOn = s.AutoUpdates };
        var focus = Number("Fokus (Min.)", s.FocusMinutes); var shortBreak = Number("Kurze Pause", s.ShortBreak); var longBreak = Number("Lange Pause", s.LongBreak); var sessions = Number("Sitzungen bis lange Pause", s.SessionsBeforeLongBreak, 1, 12);
        var workStart = Number("Arbeitsbeginn (Stunde)", s.WorkStart, 0, 23); var workEnd = Number("Arbeitsende (Stunde)", s.WorkEnd, 1, 23); var buffer = Number("Puffer (Min.)", s.BufferMinutes, 0, 60);
        var hotkey = Choice("Quick Capture: Strg + Alt +", ["Leertaste", "Q", "N"], s.Hotkey == 0x51 ? "Q" : s.Hotkey == 0x4E ? "N" : "Leertaste");
        page.Children.Add(Card(Stack(Label("Darstellung & Verhalten", 21), Row(theme, accent), startPage, tray, notification, analytics, updates, hotkey)));
        page.Children.Add(Card(Stack(Label("Fokus & Planung", 21), Row(focus, shortBreak, longBreak), sessions, Row(workStart, workEnd, buffer))));
        var widgets = Stack(Label("Dashboard", 21)); var visibility = new List<(string Id, CheckBox Check)>(); foreach (var id in new[] { "Next", "Week", "Deadlines" }) { var c = new CheckBox { Content = id switch { "Next" => "Nächster Schritt", "Week" => "Wochenübersicht", _ => "Fälligkeiten" }, IsChecked = !s.HiddenWidgets.Contains(id) }; visibility.Add((id, c)); widgets.Children.Add(c); }
        widgets.Children.Add(Button("Widget-Reihenfolge drehen", async () => { var first = s.WidgetOrder[0]; s.WidgetOrder.RemoveAt(0); s.WidgetOrder.Add(first); await vm.SaveAsync(); Show("Dashboard-Reihenfolge geändert."); })); page.Children.Add(Card(widgets));
        page.Children.Add(Button("Einstellungen speichern", async () =>
        {
            var next = Clone(s); next.Theme = theme.SelectedItem.ToString()!; next.Accent = accent.SelectedItem.ToString()!; next.StartPage = startPage.SelectedItem.ToString()!; next.MinimizeToTray = tray.IsOn; next.Notifications = notification.IsOn; next.Analytics = analytics.IsOn; next.AutoUpdates = updates.IsOn; next.FocusMinutes = (int)focus.Value; next.ShortBreak = (int)shortBreak.Value; next.LongBreak = (int)longBreak.Value; next.SessionsBeforeLongBreak = (int)sessions.Value; next.WorkStart = (int)workStart.Value; next.WorkEnd = (int)workEnd.Value; next.BufferMinutes = (int)buffer.Value; next.Hotkey = hotkey.SelectedIndex switch { 1 => 0x51u, 2 => 0x4Eu, _ => 0x20u }; next.HiddenWidgets = visibility.Where(v => v.Check.IsChecked != true).Select(v => v.Id).ToList();
            var validate = new AppData { Settings = next }; Store.Validate(validate); vm.Data.Settings = next; await vm.SaveAsync(); ApplyTheme(); native?.Register(next.Hotkey); Show("Einstellungen gespeichert."); Navigate(current);
        }, true));
        var autostart = new ToggleSwitch { Header = "Bei der Windows-Anmeldung starten" };
        using (var key = Microsoft.Win32.Registry.CurrentUser.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\Run")) autostart.IsOn = key?.GetValue("Flowstate") != null;
        autostart.Toggled += async (_, _) => await Run(() => { using var key = Microsoft.Win32.Registry.CurrentUser.CreateSubKey(@"Software\Microsoft\Windows\CurrentVersion\Run"); if (autostart.IsOn) key.SetValue("Flowstate", $"\"{Environment.ProcessPath}\""); else key.DeleteValue("Flowstate", false); return Task.CompletedTask; }); page.Children.Add(autostart);
        page.Children.Add(Card(Stack(Label("Daten & Privatsphäre", 21), Muted(vm.Store.DatabasePath), Muted("Keine Telemetrie. Exporte und Backups enthalten deine persönlichen Daten im Klartext. Bewahre sie sicher auf."), Row(Button("Backup erstellen", async () => { var path = await vm.Store.BackupAsync(vm.Data); Show($"Sicherung erstellt: {path}"); }), Button("Exportieren", ExportDialog), Button("Import / Wiederherstellen", async () =>
        {
            var path = await PickFile([".json"]); if (path == null) return; if (!await Confirm("Daten wiederherstellen?", "Der aktuelle Stand wird durch die gewählte Datei ersetzt. Vorher wird automatisch ein Backup erstellt.")) return; vm.Data = await vm.Store.ImportAsync(path, vm.Data); ApplyTheme(); Navigate(current); Show("Daten wiederhergestellt.");
        })), Button("Aktuelle Daten löschen", async () => { if (!await Confirm("Alle aktuellen Daten löschen?", "Aufgaben, Kalender und Fokushistorie werden geleert. Eine Sicherung bleibt im Backup-Ordner erhalten.")) return; await vm.Store.BackupAsync(vm.Data); vm.Data = new(); await vm.SaveAsync(); Navigate(current); }))));
        page.Children.Add(Button("Jetzt nach Updates suchen", () => CheckUpdates(true)));
    }
    async Task ExportDialog()
    {
        var picker = new Windows.Storage.Pickers.FileSavePicker { SuggestedFileName = $"Flowstate-{DateTime.Today:yyyy-MM-dd}" }; picker.FileTypeChoices.Add("Flowstate-Daten", new List<string> { ".json" }); WinRT.Interop.InitializeWithWindow.Initialize(picker, WinRT.Interop.WindowNative.GetWindowHandle(this)); var file = await picker.PickSaveFileAsync(); if (file != null) { await Store.ExportAsync(vm.Data, file.Path); Show("Daten exportiert."); }
    }
    static readonly string AppVersion = Assembly.GetExecutingAssembly().GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion.Split('+')[0] ?? "0.0.0";
    async Task CheckUpdates(bool manual)
    {
        if (!manual && (!vm.Data.Settings.AutoUpdates || vm.Data.Settings.LastUpdateCheck > DateTimeOffset.UtcNow.AddDays(-1))) return;
        try
        {
            var release = await vm.Updates.CheckAsync(AppVersion); vm.Data.Settings.LastUpdateCheck = DateTimeOffset.UtcNow; await vm.SaveAsync();
            if (release == null) { if (manual) Show("Keine neuere stabile Version verfügbar."); return; }
            Show($"Flowstate {release.Tag} ist verfügbar. Öffne „Über Flowstate“, um es zu installieren."); availableRelease = release;
        }
        catch (Exception e) { App.Log(e); if (manual) Show("Updates konnten nicht geprüft werden. Prüfe deine Internetverbindung.", true); }
    }
    ReleaseInfo? availableRelease;
    void AboutPage()
    {
        Header("Your time. Your focus. Your flow.", "Flowstate", $"Version {AppVersion} · Windows x64 · Entwicklungsstand");
        page.Children.Add(Card(Stack(Label("Lokal. Ruhig. In deinem Tempo.", 23), Muted("WinUI 3 · .NET 10 · SQLite · Keine Registrierung · Keine Telemetrie"), Button("Quellcode auf GitHub", () => OpenTarget($"https://github.com/{UpdateService.Repository}")), Button("Nach Updates suchen", async () => { await CheckUpdates(true); Navigate(current); }))));
        if (availableRelease is { } release)
            page.Children.Add(Card(Stack(Label($"Update {release.Tag}", 22), Muted(release.Notes ?? ""), Muted("Installer sind derzeit unsigniert. SHA-256 prüft die Integrität; eine Herausgebersignatur liegt nicht vor."), Button("Update herunterladen und installieren", async () =>
            {
                if (!await Confirm("Update installieren?", "Der Installer wird von GitHub geladen und per SHA-256 geprüft. Flowstate beendet sich anschließend. Windows-Sicherheitsabfragen bleiben aktiv.")) return;
                Show("Update wird heruntergeladen …"); var path = await vm.Updates.DownloadAsync(release, Path.Combine(vm.Store.DirectoryPath, "Cache")); await vm.SaveAsync();
                string mode = "/CURRENTUSER"; using (var key = Microsoft.Win32.Registry.LocalMachine.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\Uninstall\Flowstate_is1")) { if (key?.GetValue("InstallLocation") is string location && Path.GetFullPath(location).TrimEnd('\\').Equals(AppContext.BaseDirectory.TrimEnd('\\'), StringComparison.OrdinalIgnoreCase)) mode = "/ALLUSERS"; }
                var info = new ProcessStartInfo(path) { UseShellExecute = true }; info.ArgumentList.Add(mode); info.ArgumentList.Add("/DIR=" + AppContext.BaseDirectory.TrimEnd('\\')); Process.Start(info); exiting = true; Close();
            }, true))));
        page.Children.Add(Card(Stack(Label("Status dieser Version", 20), Muted("Implementiert: Aufgaben, Projekte, Tag-/Wochenraster und Monatskalender, AutoPlan-Vorschau mit Undo, Fokusprotokollierung, Noise-Mixer, Workspaces, Quick Capture, Auswertungen, lokale Backups und Update-Prüfung."), Muted("Noch offen: vollständige Mehrsprachigkeit, komplexe Serienregeln (z. B. letzter Werktag), lizenzierte Naturklänge, automatische Fensteranordnung und vollständige Windows-11-Abnahme. Details im Entwicklungsstatus auf GitHub."))));
    }
}
