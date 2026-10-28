using CommunityToolkit.Mvvm.ComponentModel;
using Flowstate.Core;
using Flowstate.Infrastructure;
namespace Flowstate.App;

public partial class MainViewModel(Store store, UpdateService updates) : ObservableObject
{
    public Store Store { get; } = store;
    public UpdateService Updates { get; } = updates;
    public AppData Data { get; set; } = new();
    [ObservableProperty] public partial bool Busy { get; set; }
    [ObservableProperty] public partial string Status { get; set; } = "Alles lokal. Alles in deinem Tempo.";
    public async Task LoadAsync() { Busy = true; try { Data = await Store.LoadAsync(); } finally { Busy = false; } }
    public async Task SaveAsync()
    {
        try { await Store.SaveAsync(Data); }
        catch { Data = await Store.LoadAsync(); throw; }
    }
    public async Task CompleteAsync(TaskItem task)
    {
        var next = task.Complete(DateTimeOffset.Now); if (next != null) Data.Tasks.Add(next);
        await SaveAsync();
    }
    public async Task DeleteAsync(TaskItem task)
    {
        Data.Tasks.Remove(task); Data.Blocks.RemoveAll(b => b.TaskId == task.Id);
        foreach (var session in Data.Sessions.Where(s => s.TaskId == task.Id)) session.TaskId = null;
        foreach (var t in Data.Tasks) t.Dependencies.Remove(task.Id);
        if (Data.ActiveTimer?.TaskId == task.Id) Data.ActiveTimer.TaskId = null;
        Data.PlanUndo = null; await SaveAsync();
    }
    public IEnumerable<TaskItem> OpenTasks => Data.Tasks.Where(t => !t.Archived && t.Status is Core.TaskStatus.Inbox or Core.TaskStatus.Todo or Core.TaskStatus.InProgress);
    public async Task FinishFocusAsync(bool completed)
    {
        var clock = Data.ActiveTimer; if (clock == null) return;
        var now = DateTimeOffset.UtcNow; clock.Pause(now);
        var minutes = (clock.Stopwatch ? clock.Elapsed(now) : Math.Min(clock.TargetSeconds, clock.Elapsed(now))) / 60;
        if (!clock.IsBreak && minutes > 0 && !Data.Sessions.Any(s => s.Id == clock.SessionId))
        {
            Data.Sessions.Add(new() { Id = clock.SessionId, TaskId = clock.TaskId, Start = clock.Created, End = now, Minutes = minutes, Completed = completed });
            var task = Data.Tasks.FirstOrDefault(t => t.Id == clock.TaskId); if (task != null) task.ActualMinutes += minutes;
        }
        Data.ActiveTimer = null; await SaveAsync();
    }
}
