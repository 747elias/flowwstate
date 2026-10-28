namespace Flowstate.Core;

public enum TaskStatus { Inbox, Todo, InProgress, Completed, OnHold, Cancelled }
public enum Priority { Urgent, High, Medium, Low }
public enum Energy { Low, Medium, High }
public enum Repeat { None, Daily, Weekly, Monthly, Yearly }
public enum BlockKind { Event, Manual, AutoPlan, Break }
public class TaskItem
{
    public Guid Id { get; set; } = Guid.NewGuid();
    public string Title { get; set; } = "";
    public string Description { get; set; } = "";
    public DateTimeOffset Created { get; set; } = DateTimeOffset.UtcNow;
    public DateTimeOffset Updated { get; set; } = DateTimeOffset.UtcNow;
    public TaskStatus Status { get; set; } = TaskStatus.Inbox;
    public Priority Priority { get; set; } = Priority.Medium;
    public Guid? ProjectId { get; set; }
    public DateTimeOffset? Due { get; set; }
    public DateTimeOffset? NotBefore { get; set; }
    public DateTimeOffset? CompletedAt { get; set; }
    public int Minutes { get; set; } = 25;
    public double ActualMinutes { get; set; }
    public bool Splittable { get; set; } = true;
    public int MinimumMinutes { get; set; } = 15;
    public Energy Energy { get; set; } = Energy.Medium;
    public Repeat Repeat { get; set; }
    public bool Archived { get; set; }
    public List<string> Tags { get; set; } = [];
    public List<string> Contexts { get; set; } = [];
    public List<Guid> Dependencies { get; set; } = [];
    public List<string> Checklist { get; set; } = [];
    public List<string> CheckedItems { get; set; } = [];
    public void Validate()
    {
        if (string.IsNullOrWhiteSpace(Title) || Title.Length > 500) throw new ArgumentException("Der Titel muss 1–500 Zeichen enthalten.");
        if (Minutes is < 1 or > 10080 || MinimumMinutes < 1 || MinimumMinutes > Minutes) throw new ArgumentException("Dauer und Mindestdauer sind ungültig.");
        if (Dependencies.Contains(Id)) throw new ArgumentException("Eine Aufgabe kann nicht von sich selbst abhängen.");
    }
    public TaskItem? Complete(DateTimeOffset now)
    {
        if (Status == TaskStatus.Completed) return null;
        Status = TaskStatus.Completed; CompletedAt = now; Updated = now;
        if (Repeat == Repeat.None) return null;
        var date = Due ?? now;
        DateTimeOffset Next(DateTimeOffset d) => Repeat switch { Repeat.Daily => d.AddDays(1), Repeat.Weekly => d.AddDays(7), Repeat.Monthly => d.AddMonths(1), _ => d.AddYears(1) };
        do { date = Next(date); } while (date <= now);
        return new TaskItem { Title = Title, Description = Description, ProjectId = ProjectId, Priority = Priority, Minutes = Minutes, MinimumMinutes = MinimumMinutes, Splittable = Splittable, Energy = Energy, Repeat = Repeat, Due = date, Status = TaskStatus.Todo, Tags = [..Tags], Contexts = [..Contexts], Checklist = [..Checklist] };
    }
}
public class Project
{
    public Guid Id { get; set; } = Guid.NewGuid();
    public string Name { get; set; } = "";
    public string Description { get; set; } = "";
    public string Color { get; set; } = "#8B80FF";
    public DateTimeOffset? Due { get; set; }
    public bool Archived { get; set; }
}
public class CalendarBlock
{
    public Guid Id { get; set; } = Guid.NewGuid();
    public Guid? TaskId { get; set; }
    public string Title { get; set; } = "";
    public DateTimeOffset Start { get; set; }
    public DateTimeOffset End { get; set; }
    public BlockKind Kind { get; set; }
    public bool Locked { get; set; }
    public Guid? PlanId { get; set; }
    public string Reason { get; set; } = "";
    public bool Overlaps(CalendarBlock b) => Start < b.End && b.Start < End;
}
public class FocusSession
{
    public Guid Id { get; set; } = Guid.NewGuid();
    public Guid? TaskId { get; set; }
    public DateTimeOffset Start { get; set; }
    public DateTimeOffset End { get; set; }
    public double Minutes { get; set; }
    public bool Completed { get; set; }
}
public class Workspace
{
    public Guid Id { get; set; } = Guid.NewGuid();
    public string Name { get; set; } = "";
    public string Description { get; set; } = "";
    public List<string> Targets { get; set; } = [];
    public int FocusMinutes { get; set; } = 25;
    public Guid? ProjectId { get; set; }
}
public class DailyReview
{
    public Guid Id { get; set; } = Guid.NewGuid();
    public DateOnly Date { get; set; }
    public string Notes { get; set; } = "";
}
public class Preferences
{
    public string Theme { get; set; } = "Dark";
    public string Accent { get; set; } = "#8B80FF";
    public string StartPage { get; set; } = "Dashboard";
    public bool MinimizeToTray { get; set; } = true;
    public bool Notifications { get; set; } = true;
    public bool Analytics { get; set; } = true;
    public bool AutoUpdates { get; set; } = true;
    public DateTimeOffset? LastUpdateCheck { get; set; }
    public int FocusMinutes { get; set; } = 25;
    public int ShortBreak { get; set; } = 5;
    public int LongBreak { get; set; } = 15;
    public int SessionsBeforeLongBreak { get; set; } = 4;
    public int WorkStart { get; set; } = 9;
    public int WorkEnd { get; set; } = 18;
    public int BufferMinutes { get; set; } = 5;
    public uint Hotkey { get; set; } = 0x20;
    public double WhiteVolume { get; set; }
    public double BrownVolume { get; set; }
    public double PinkVolume { get; set; }
    public double MasterVolume { get; set; } = .5;
    public List<string> HiddenWidgets { get; set; } = [];
    public List<string> WidgetOrder { get; set; } = ["Next", "Week", "Deadlines"];
}
public class AppData
{
    public int SchemaVersion { get; set; } = 1;
    public List<TaskItem> Tasks { get; set; } = [];
    public List<Project> Projects { get; set; } = [];
    public List<CalendarBlock> Blocks { get; set; } = [];
    public List<FocusSession> Sessions { get; set; } = [];
    public List<Workspace> Workspaces { get; set; } = [];
    public List<DailyReview> Reviews { get; set; } = [];
    public Preferences Settings { get; set; } = new();
    public FocusClock? ActiveTimer { get; set; }
    public List<CalendarBlock>? PlanUndo { get; set; }
}
