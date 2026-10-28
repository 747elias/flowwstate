namespace Flowstate.Core;

public record TimeWindow(DateTimeOffset Start, DateTimeOffset End);
public record PlanOptions(IReadOnlyList<TimeWindow> Windows, string Context = "", Energy Energy = Energy.High, int BreakMinutes = 5, int BufferMinutes = 0);
public record Unplanned(Guid TaskId, string Reason);
public record PlanResult(List<CalendarBlock> Blocks, List<Unplanned> Unplanned);
public static class Scheduler
{
    public static List<TimeWindow> FreeWindows(IEnumerable<TimeWindow> windows, IEnumerable<CalendarBlock> busy)
    {
        var result = new List<TimeWindow>();
        foreach (var w in windows.OrderBy(w => w.Start))
        {
            if (w.End <= w.Start) throw new ArgumentException("Das Ende muss nach dem Beginn liegen.");
            if (result.Count > 0 && result[^1].End > w.Start) throw new ArgumentException("Verfügbarkeiten dürfen nicht überlappen.");
            var cursor = w.Start;
            foreach (var b in busy.Where(b => b.End > w.Start && b.Start < w.End).OrderBy(b => b.Start))
            {
                if (b.Start > cursor) result.Add(new(cursor, b.Start));
                if (b.End > cursor) cursor = b.End;
            }
            if (cursor < w.End) result.Add(new(cursor, w.End));
        }
        return result;
    }
    public static PlanResult Plan(IReadOnlyList<TaskItem> tasks, IReadOnlyList<CalendarBlock> existing, PlanOptions options)
    {
        if (options.BreakMinutes < 0 || options.BufferMinutes < 0) throw new ArgumentException("Pausen dürfen nicht negativ sein.");
        var blocks = new List<CalendarBlock>(); var planId = Guid.NewGuid();
        var active = tasks.Where(t => !t.Archived && t.Status is TaskStatus.Inbox or TaskStatus.Todo or TaskStatus.InProgress).ToList();
        foreach (var task in active) task.Validate();
        var remaining = active.ToDictionary(t => t.Id, t => Math.Max(0, t.Minutes - (int)Math.Floor(t.ActualMinutes) - (int)existing.Where(b => b.TaskId == t.Id && b.Kind != BlockKind.Break).Sum(b => (b.End - b.Start).TotalMinutes)));
        var finished = tasks.Where(t => t.Status == TaskStatus.Completed).ToDictionary(t => t.Id, t => DateTimeOffset.MinValue);
        foreach (var t in active.Where(t => remaining[t.Id] == 0))
            finished[t.Id] = existing.Where(b => b.TaskId == t.Id).Select(b => b.End).DefaultIfEmpty(DateTimeOffset.MinValue).Max();
        Guid? previousProject = null;
        foreach (var slot in FreeWindows(options.Windows, existing))
        {
            var cursor = slot.Start;
            while (cursor < slot.End)
            {
                var candidates = active.Where(t => remaining[t.Id] > 0 && t.Energy <= options.Energy &&
                    (string.IsNullOrWhiteSpace(options.Context) || t.Contexts.Count == 0 || t.Contexts.Contains(options.Context, StringComparer.OrdinalIgnoreCase)) &&
                    t.Dependencies.All(finished.ContainsKey))
                    .Select(t => new { Task = t, Start = new[] { cursor, t.NotBefore ?? cursor, t.Dependencies.Select(id => finished[id]).DefaultIfEmpty(cursor).Max() }.Max() })
                    .Where(x => x.Start < slot.End && (slot.End - x.Start).TotalMinutes >= (x.Task.Splittable ? Math.Min(remaining[x.Task.Id], x.Task.MinimumMinutes) : remaining[x.Task.Id]))
                    .OrderByDescending(x => Score(x.Task, x.Start, previousProject)).ThenBy(x => x.Task.Created).ThenBy(x => x.Task.Id).ToList();
                if (candidates.Count == 0) break;
                var chosen = candidates[0]; var task = chosen.Task; cursor = chosen.Start;
                var capacity = (int)Math.Floor((slot.End - cursor).TotalMinutes);
                var amount = task.Splittable ? Math.Min(remaining[task.Id], capacity) : remaining[task.Id];
                // Avoid leaving a fragment smaller than the task's minimum session length.
                if (remaining[task.Id] > amount && remaining[task.Id] - amount < task.MinimumMinutes) amount = remaining[task.Id] - task.MinimumMinutes;
                if (amount < Math.Min(task.MinimumMinutes, remaining[task.Id])) break;
                var end = cursor.AddMinutes(amount);
                blocks.Add(new() { TaskId = task.Id, Title = task.Title, Start = cursor, End = end, Kind = BlockKind.AutoPlan, PlanId = planId,
                    Reason = $"Priorität {task.Priority} · {amount} Min." + (task.Due is {} d ? $" · Fällig {d.ToLocalTime():dd.MM. HH:mm}" : "") + (task.Due < end ? " · Achtung: nach Fälligkeit" : "") });
                remaining[task.Id] -= amount; if (remaining[task.Id] == 0) finished[task.Id] = end;
                previousProject = task.ProjectId; cursor = end;
                var pause = options.BreakMinutes + options.BufferMinutes;
                if (pause > 0 && cursor.AddMinutes(pause) <= slot.End)
                { blocks.Add(new() { Title = "Pause & Puffer", Start = cursor, End = cursor.AddMinutes(pause), Kind = BlockKind.Break, PlanId = planId }); cursor = cursor.AddMinutes(pause); }
                else if (pause > 0) break;
            }
        }
        var unplanned = active.Where(t => remaining[t.Id] > 0).Select(t => new Unplanned(t.Id,
            t.Energy > options.Energy ? "Energiebedarf zu hoch" :
            !string.IsNullOrWhiteSpace(options.Context) && t.Contexts.Count > 0 && !t.Contexts.Contains(options.Context, StringComparer.OrdinalIgnoreCase) ? "Kontext passt nicht" :
            t.Dependencies.Any(id => !finished.ContainsKey(id)) ? "Offene oder zyklische Abhängigkeit" :
            $"{remaining[t.Id]} Min. verbleiben: kein passendes freies Zeitfenster")).ToList();
        return new(blocks, unplanned);
    }
    static double Score(TaskItem t, DateTimeOffset start, Guid? project) => (4 - (int)t.Priority) * 100 +
        (t.Due is {} due ? Math.Clamp(200 - (due - start).TotalHours * 3, -100, 400) : 0) +
        (t.Status == TaskStatus.InProgress ? 35 : 0) + (project.HasValue && t.ProjectId == project ? 15 : 0);

    public static TimeWindow LocalWindow(DateOnly day, TimeOnly start, TimeOnly end, TimeZoneInfo zone)
    {
        DateTimeOffset Convert(TimeOnly time)
        {
            var local = day.ToDateTime(time, DateTimeKind.Unspecified);
            if (zone.IsInvalidTime(local)) throw new ArgumentException("Diese Uhrzeit existiert wegen der Zeitumstellung nicht.");
            if (zone.IsAmbiguousTime(local)) throw new ArgumentException("Diese Uhrzeit ist wegen der Zeitumstellung mehrdeutig.");
            return new(local, zone.GetUtcOffset(local));
        }
        var window = new TimeWindow(Convert(start), Convert(end));
        if (window.End <= window.Start) throw new ArgumentException("Arbeitsende muss nach Arbeitsbeginn liegen.");
        return window;
    }
}
