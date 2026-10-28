using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Flowstate.Core;
using Flowstate.Infrastructure;
using Status = Flowstate.Core.TaskStatus;
namespace Flowstate.App;

public sealed partial class MainWindow
{
    void Dashboard()
    {
        Header(DateTime.Today.ToString("dddd, dd. MMMM", System.Globalization.CultureInfo.GetCultureInfo("de-CH")), "Raum für das Wesentliche.", "Ein klarer Plan. Ein Schritt nach dem anderen.");
        var capture = Input("", ""); capture.PlaceholderText = "Was möchtest du erledigen? z. B. Lesen morgen für 30 Minuten"; capture.MinWidth = 320;
        page.Children.Add(Card(Stack(Label("Gedanken festhalten", 18), capture, Button("+ Aufgabe erfassen", async () => { vm.Data.Tasks.Add(Capture.Parse(capture.Text, DateTimeOffset.Now)); await vm.SaveAsync(); Navigate(current); }, true))));
        var today = DateTime.Today; var focus = vm.Data.Sessions.Where(s => s.Start.ToLocalTime().Date == today).Sum(s => s.Minutes);
        var stats = new Grid { ColumnSpacing = 16 }; for (int i = 0; i < 3; i++) stats.ColumnDefinitions.Add(new ColumnDefinition());
        var values = new[] { (vm.OpenTasks.Count().ToString(), "Offene Aufgaben"), ($"{focus:0} min", "Fokus heute"), (vm.Data.Tasks.Count(t => t.CompletedAt?.ToLocalTime().Date == today).ToString(), "Heute erledigt") };
        for (int i = 0; i < 3; i++) { var c = Card(Stack(Label(values[i].Item1, 30), Muted(values[i].Item2))); Grid.SetColumn(c, i); stats.Children.Add(c); }
        page.Children.Add(stats);
        foreach (var widget in vm.Data.Settings.WidgetOrder.Where(w => !vm.Data.Settings.HiddenWidgets.Contains(w)))
        {
            if (widget == "Next")
            {
                var next = vm.Data.Blocks.Where(b => b.End > DateTimeOffset.Now && b.TaskId.HasValue).OrderBy(b => b.Start).FirstOrDefault();
                var task = vm.OpenTasks.FirstOrDefault(t => t.Id == next?.TaskId) ?? vm.OpenTasks.OrderBy(t => t.Priority).ThenBy(t => t.Due).FirstOrDefault();
                page.Children.Add(Card(Stack(Muted("DEIN NÄCHSTER SCHRITT"), Label(task?.Title ?? "Platz für einen neuen Anfang", 25), Muted(next == null ? "Plane deinen Tag passend zu deiner verfügbaren Zeit." : When(next.Start)), task == null ? Go("Tag planen", "AutoPlan") : Button("Fokus starten →", () => StartFocus(task.Id, task.Minutes), true))));
            }
            if (widget == "Week") { page.Children.Add(Card(Stack(Label("Deine Woche", 20), WeekChart(7), Go("Auswertung öffnen", "Analytics")))); }
            if (widget == "Deadlines")
            {
                var box = Stack(Label("Anstehende Fälligkeiten", 20)); foreach (var t in vm.OpenTasks.Where(t => t.Due.HasValue).OrderBy(t => t.Due).Take(4)) box.Children.Add(TaskRow(t));
                if (box.Children.Count == 1) box.Children.Add(Muted("Keine offenen Fälligkeiten. Zeit zum Durchatmen.")); page.Children.Add(Card(box));
            }
        }
        page.Children.Add(Row(Go("Plan My Day", "AutoPlan"), Go("Tagesrückblick", "Tagesrückblick"), Go("Fokus öffnen", "Focus Studio")));
    }
    UIElement TaskRow(TaskItem task)
    {
        var grid = new Grid { ColumnSpacing = 14, Padding = new Thickness(0, 8, 0, 8) }; grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto }); grid.ColumnDefinitions.Add(new ColumnDefinition()); grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        var check = new CheckBox { IsChecked = task.Status == Status.Completed, VerticalAlignment = VerticalAlignment.Center }; Microsoft.UI.Xaml.Automation.AutomationProperties.SetName(check, $"{task.Title} abschließen");
        check.Click += async (_, _) => await Run(async () => { if (check.IsChecked == true) await vm.CompleteAsync(task); else { task.Status = Status.Todo; task.CompletedAt = null; await vm.SaveAsync(); } Navigate(current); });
        grid.Children.Add(check);
        var detail = Stack(Label(task.Title, 16), Muted($"{task.Priority} · {task.Minutes} Min. · {task.Status}" + (task.Due is { } d ? $" · {When(d)}" : "") + (task.ProjectId is { } id ? $" · {vm.Data.Projects.FirstOrDefault(p => p.Id == id)?.Name}" : "")));
        detail.Spacing = 4; Grid.SetColumn(detail, 1); grid.Children.Add(detail);
        var more = new DropDownButton { Content = "•••" }; var menu = new MenuFlyout();
        void Action(string name, Func<Task> run) { var i = new MenuFlyoutItem { Text = name }; i.Click += async (_, _) => await Run(run); menu.Items.Add(i); }
        Action("Fokus starten", () => StartFocus(task.Id, task.Minutes));
        Action("Duplizieren", async () => { var copy = Clone(task); copy.Id = Guid.NewGuid(); copy.Title += " (Kopie)"; copy.Status = Status.Todo; copy.CompletedAt = null; copy.ActualMinutes = 0; copy.CheckedItems = []; copy.NextOccurrenceId = null; vm.Data.Tasks.Add(copy); await vm.SaveAsync(); Navigate(current); });
        Action(task.Archived ? "Aus Archiv holen" : "Archivieren", async () => { task.Archived = !task.Archived; await vm.SaveAsync(); Navigate(current); });
        Action("Löschen", async () => { if (await Confirm("Aufgabe löschen?", task.Title)) { await vm.DeleteAsync(task); Navigate(current); } }); more.Flyout = menu;
        var edit = Row(Button("Öffnen", () => TaskDialog(task)), more); Grid.SetColumn(edit, 2); grid.Children.Add(edit); return grid;
    }
    Guid? projectFilter;
    void TasksPage()
    {
        Header("Capture → Organize", "Deine Aufgaben", "Alles an einem Ort. Sortiert nach dem, was jetzt wichtig ist.");
        var search = Input("Suche"); search.PlaceholderText = "Titel, Notizen, Tags …";
        var filter = Choice("Ansicht", ["Offen", "Alle", "Heute", "Diese Woche", "Überfällig", "Inbox", "Geplant", "Abgeschlossen", "Archiv"]);
        var project = Choice("Projekt", new[] { "Alle Projekte" }.Concat(vm.Data.Projects.Select(p => p.Name)), vm.Data.Projects.FirstOrDefault(p => p.Id == projectFilter)?.Name);
        var sort = Choice("Sortierung", ["Priorität", "Fälligkeit", "Titel", "Status", "Kontext"]);
        var list = Stack(); var selected = new HashSet<Guid>(); var selectionLabel = Muted("0 Aufgaben ausgewählt");
        void Refresh()
        {
            list.Children.Clear(); var now = DateTimeOffset.Now; IEnumerable<TaskItem> tasks = vm.Data.Tasks;
            tasks = filter.SelectedIndex switch { 0 => vm.OpenTasks, 2 => tasks.Where(t => t.Due?.ToLocalTime().Date == DateTime.Today), 3 => tasks.Where(t => t.Due >= now && t.Due < now.AddDays(7)), 4 => vm.OpenTasks.Where(t => t.Due < now), 5 => tasks.Where(t => t.Status == Status.Inbox), 6 => tasks.Where(t => vm.Data.Blocks.Any(b => b.TaskId == t.Id)), 7 => tasks.Where(t => t.Status == Status.Completed), 8 => tasks.Where(t => t.Archived), _ => tasks };
            if (filter.SelectedIndex != 8) tasks = tasks.Where(t => !t.Archived);
            if (project.SelectedIndex > 0) { var id = vm.Data.Projects[project.SelectedIndex - 1].Id; tasks = tasks.Where(t => t.ProjectId == id); }
            tasks = tasks.Where(t => $"{t.Title} {t.Description} {string.Join(' ', t.Tags)}".Contains(search.Text, StringComparison.OrdinalIgnoreCase));
            tasks = sort.SelectedIndex switch { 1 => tasks.OrderBy(t => t.Due ?? DateTimeOffset.MaxValue), 2 => tasks.OrderBy(t => t.Title), 3 => tasks.OrderBy(t => t.Status), 4 => tasks.OrderBy(t => string.Join(',', t.Contexts)), _ => tasks.OrderBy(t => t.Priority).ThenBy(t => t.Due) };
            selected.Clear(); selectionLabel.Text = "0 Aufgaben ausgewählt";
            foreach (var t in tasks)
            {
                var select = new CheckBox { Content = "Auswählen" }; select.Click += (_, _) => { if (select.IsChecked == true) selected.Add(t.Id); else selected.Remove(t.Id); selectionLabel.Text = $"{selected.Count} Aufgaben ausgewählt"; }; list.Children.Add(Card(Stack(select, TaskRow(t))));
            }
            if (list.Children.Count == 0) list.Children.Add(Card(Stack(Label("Ein freier Kopf beginnt hier.", 20), Muted("Erstelle eine Aufgabe oder passe die Filter an."))));
        }
        search.TextChanged += (_, _) => Refresh(); filter.SelectionChanged += (_, _) => Refresh(); project.SelectionChanged += (_, _) => Refresh(); sort.SelectionChanged += (_, _) => Refresh();
        page.Children.Add(Row(Button("+ Neue Aufgabe", () => TaskDialog(null), true), Button("Schnellerfassung", CaptureDialog)));
        page.Children.Add(search); page.Children.Add(Row(filter, project, sort));
        page.Children.Add(Row(selectionLabel, Button("Auswahl abschließen", async () => { foreach (var t in vm.Data.Tasks.Where(t => selected.Contains(t.Id)).ToList()) { var next = t.Complete(DateTimeOffset.Now); if (next != null) vm.Data.Tasks.Add(next); } await vm.SaveAsync(); Refresh(); }), Button("Auswahl archivieren", async () => { foreach (var t in vm.Data.Tasks.Where(t => selected.Contains(t.Id))) t.Archived = true; await vm.SaveAsync(); Refresh(); }), Button("Auswahl löschen", async () => { var chosen = vm.Data.Tasks.Where(t => selected.Contains(t.Id)).ToList(); if (chosen.Count == 0) return; if (await Confirm($"{chosen.Count} Aufgaben löschen?", string.Join("\n", chosen.Select(t => t.Title)))) { foreach (var t in chosen) await vm.DeleteAsync(t); Refresh(); } })));
        page.Children.Add(list); Refresh();
    }
    async Task CaptureDialog()
    {
        ShowWindow(); var entry = Input("Aufgabe · Datum und Dauer werden lokal erkannt"); entry.PlaceholderText = "Netzwerk lernen am Freitag für 45 Minuten";
        if (!await Edit("Gedanken festhalten", Stack(entry), "Weiter")) return;
        var parsed = Capture.Parse(entry.Text, DateTimeOffset.Now); await TaskDialog(parsed, true);
    }
    async Task TaskDialog(TaskItem? original, bool isNew = false)
    {
        var t = original == null ? new TaskItem() : Clone(original); isNew |= original == null;
        var title = Input("Titel", t.Title); var description = Input("Notizen", t.Description, true); var duration = Number("Dauer (Minuten)", t.Minutes, 1, 10080);
        var priority = Choice("Priorität", Enum.GetNames<Priority>(), t.Priority.ToString()); var status = Choice("Status", Enum.GetNames<Status>(), t.Status.ToString());
        var projectList = vm.Data.Projects.Where(p => !p.Archived).ToList(); var project = Choice("Projekt", new[] { "Kein Projekt" }.Concat(projectList.Select(p => p.Name)), projectList.FirstOrDefault(p => p.Id == t.ProjectId)?.Name);
        var due = new CalendarDatePicker { Header = "Fällig am", Date = t.Due }; var start = new CalendarDatePicker { Header = "Frühester Beginn", Date = t.NotBefore };
        var split = new CheckBox { Content = "In mehrere Sitzungen aufteilbar", IsChecked = t.Splittable }; var minimum = Number("Mindestdauer", t.MinimumMinutes);
        var energy = Choice("Energiebedarf", Enum.GetNames<Energy>(), t.Energy.ToString()); var repeat = Choice("Wiederholung", Enum.GetNames<Repeat>(), t.Repeat.ToString());
        var interval = Number("Wiederholung CustomDays: alle N Tage", t.RepeatIntervalDays, 1, 3650);
        var tags = Input("Tags (Komma getrennt)", string.Join(", ", t.Tags)); var context = Input("Kontexte (Komma getrennt)", string.Join(", ", t.Contexts));
        var checklist = Input("Checkliste (eine Zeile je Schritt)", string.Join('\n', t.Checklist), true);
        var fields = Stack(title, description, Row(duration, priority), Row(status, project), Row(due, start), Row(energy, repeat), interval, split, minimum, tags, context, checklist);
        var dependencies = new List<(CheckBox Check, Guid Id)>();
        var dependencyPanel = Stack(); foreach (var candidate in vm.OpenTasks.Where(x => x.Id != t.Id)) { var c = new CheckBox { Content = candidate.Title, IsChecked = t.Dependencies.Contains(candidate.Id) }; dependencies.Add((c, candidate.Id)); dependencyPanel.Children.Add(c); }
        fields.Children.Add(new Expander { Header = "Abhängigkeiten", Content = dependencyPanel, HorizontalAlignment = HorizontalAlignment.Stretch });
        var checks = new List<CheckBox>(); foreach (var step in t.Checklist) { var c = new CheckBox { Content = step, IsChecked = t.CheckedItems.Contains(step) }; checks.Add(c); fields.Children.Add(c); }
        if (!await Edit(isNew ? "Neue Aufgabe" : "Aufgabe bearbeiten", fields)) return;
        t.Title = title.Text.Trim(); t.Description = description.Text; t.Minutes = (int)duration.Value; t.MinimumMinutes = Math.Min((int)minimum.Value, t.Minutes); t.Priority = Enum.Parse<Priority>(priority.SelectedItem.ToString()!); t.Status = Enum.Parse<Status>(status.SelectedItem.ToString()!);
        t.ProjectId = project.SelectedIndex > 0 ? projectList[project.SelectedIndex - 1].Id : null;
        t.Due = due.Date is { } dd ? new DateTimeOffset(dd.Date.AddHours(18), TimeZoneInfo.Local.GetUtcOffset(dd.Date)) : null; t.NotBefore = start.Date;
        t.Splittable = split.IsChecked == true; t.Energy = Enum.Parse<Energy>(energy.SelectedItem.ToString()!); t.Repeat = Enum.Parse<Repeat>(repeat.SelectedItem.ToString()!); t.RepeatIntervalDays = (int)interval.Value;
        t.Tags = tags.Text.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries).Distinct().ToList(); t.Contexts = context.Text.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries).Distinct().ToList();
        t.Dependencies = dependencies.Where(x => x.Check.IsChecked == true).Select(x => x.Id).ToList();
        t.Checklist = checklist.Text.Split('\n', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries).Distinct().ToList(); t.CheckedItems = checks.Where(c => c.IsChecked == true).Select(c => c.Content.ToString()!).Where(t.Checklist.Contains).ToList(); t.Updated = DateTimeOffset.Now; t.Validate();
        if (!isNew) vm.Data.Tasks.Remove(original!);
        var completed = t.Status == Status.Completed && original?.Status != Status.Completed;
        if (completed) t.Status = Status.Todo;
        vm.Data.Tasks.Add(t); if (completed) { var next = t.Complete(DateTimeOffset.Now); if (next != null) vm.Data.Tasks.Add(next); }
        await vm.SaveAsync(); Navigate(current);
    }
    void ProjectsPage()
    {
        Header("Organize", "Projekte", "Gib deinen Aufgaben eine Richtung."); page.Children.Add(Button("+ Projekt erstellen", () => ProjectDialog(null), true));
        foreach (var p in vm.Data.Projects)
        {
            var tasks = vm.Data.Tasks.Where(t => t.ProjectId == p.Id).ToList(); var completed = tasks.Count(t => t.Status == Status.Completed); var percentage = tasks.Count == 0 ? 0 : 100.0 * completed / tasks.Count;
            page.Children.Add(Card(Stack(Label(p.Name + (p.Archived ? " · Archiv" : ""), 22), Muted(p.Description), new ProgressBar { Value = percentage, Maximum = 100 }, Muted($"{completed} von {tasks.Count} erledigt · {tasks.Sum(t => t.Minutes)} Min. geplant · {tasks.Sum(t => t.ActualMinutes):0} Min. geleistet"), Row(Button("Bearbeiten", () => ProjectDialog(p)), Button("Aufgaben", () => { projectFilter = p.Id; Navigate("Aufgaben"); return Task.CompletedTask; }), Button("Löschen", async () => { if (await Confirm("Projekt löschen?", "Die zugehörigen Aufgaben bleiben ohne Projekt erhalten.")) { foreach (var t in vm.Data.Tasks.Where(t => t.ProjectId == p.Id)) t.ProjectId = null; vm.Data.Projects.Remove(p); await vm.SaveAsync(); Navigate(current); } })))));
        }
        if (vm.Data.Projects.Count == 0) Empty("Von einer Idee zum Projekt", "Lege dein erstes Projekt an und ordne ihm Aufgaben zu.");
    }
    async Task ProjectDialog(Project? original)
    {
        var p = original == null ? new Project() : Clone(original); var name = Input("Name", p.Name); var description = Input("Beschreibung", p.Description, true); var due = new CalendarDatePicker { Header = "Zieldatum", Date = p.Due }; var archived = new CheckBox { Content = "Archiviert", IsChecked = p.Archived };
        if (!await Edit("Projekt", Stack(name, description, due, archived))) return;
        if (string.IsNullOrWhiteSpace(name.Text)) throw new ArgumentException("Ein Projektname ist erforderlich."); p.Name = name.Text.Trim(); p.Description = description.Text; p.Due = due.Date; p.Archived = archived.IsChecked == true;
        if (original != null) vm.Data.Projects.Remove(original); vm.Data.Projects.Add(p); await vm.SaveAsync(); Navigate(current);
    }
}
