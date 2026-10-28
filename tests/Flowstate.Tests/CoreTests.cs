using Flowstate.Core;
using Flowstate.Infrastructure;
using Status = Flowstate.Core.TaskStatus;
namespace Flowstate.Tests;

public class CoreTests
{
    static readonly DateTimeOffset Day = new(2026, 10, 28, 9, 0, 0, TimeSpan.Zero);
    static TaskItem Task(string title = "Test", int minutes = 30) => new() { Title = title, Minutes = minutes, MinimumMinutes = Math.Min(15, minutes) };
    static PlanOptions Window(int minutes = 180) => new([new(Day, Day.AddMinutes(minutes))], BreakMinutes: 0);
    [Fact] public void EmptyPlan() { Assert.Empty(Scheduler.Plan([], [], Window()).Blocks); }
    [Fact] public void UrgentWins() { var low = Task("low"); low.Priority = Priority.Low; var urgent = Task("urgent"); urgent.Priority = Priority.Urgent; Assert.Equal(urgent.Id, Scheduler.Plan([low, urgent], [], Window()).Blocks[0].TaskId); }
    [Fact] public void BusyCalendarIsRespected() { var t = Task(); var busy = new CalendarBlock { Start = Day, End = Day.AddHours(1) }; var p = Scheduler.Plan([t], [busy], Window()); Assert.All(p.Blocks, b => Assert.False(b.Overlaps(busy))); Assert.Equal(Day.AddHours(1), p.Blocks[0].Start); }
    [Fact] public void FullCalendar() { var p = Scheduler.Plan([Task()], [new() { Start = Day, End = Day.AddHours(3) }], Window()); Assert.Empty(p.Blocks); Assert.Single(p.Unplanned); }
    [Fact] public void UnsplittableTaskDoesNotFit() { var t = Task(minutes: 120); t.Splittable = false; Assert.Empty(Scheduler.Plan([t], [], Window(60)).Blocks); }
    [Fact] public void SplitTaskAcrossDays() { var t = Task(minutes: 120); var p = Scheduler.Plan([t], [], new([new(Day, Day.AddMinutes(60)), new(Day.AddDays(1), Day.AddDays(1).AddMinutes(60))], BreakMinutes: 0)); Assert.Equal(2, p.Blocks.Count); Assert.Empty(p.Unplanned); }
    [Fact] public void SmallRemainderIsAvoided() { var t = Task(minutes: 70); t.MinimumMinutes = 20; var p = Scheduler.Plan([t], [], Window(60)); Assert.Equal(50, (p.Blocks[0].End - p.Blocks[0].Start).TotalMinutes); }
    [Fact] public void StartConstraint() { var t = Task(); t.NotBefore = Day.AddHours(1); Assert.Equal(t.NotBefore, Scheduler.Plan([t], [], Window()).Blocks[0].Start); }
    [Fact] public void ContextConstraint() { var t = Task(); t.Contexts = ["Büro"]; Assert.Empty(Scheduler.Plan([t], [], Window() with { Context = "Zuhause" }).Blocks); }
    [Fact] public void EnergyConstraint() { var t = Task(); t.Energy = Energy.High; Assert.Empty(Scheduler.Plan([t], [], Window() with { Energy = Energy.Low }).Blocks); }
    [Fact] public void DependencyOrder() { var a = Task("a"); a.Priority = Priority.Low; var b = Task("b"); b.Priority = Priority.Urgent; b.Dependencies = [a.Id]; var p = Scheduler.Plan([b, a], [], Window()); Assert.Equal(a.Id, p.Blocks[0].TaskId); Assert.True(p.Blocks[1].Start >= p.Blocks[0].End); }
    [Fact] public void CyclesReported() { var a = Task(); var b = Task(); a.Dependencies = [b.Id]; b.Dependencies = [a.Id]; var p = Scheduler.Plan([a, b], [], Window()); Assert.Empty(p.Blocks); Assert.Equal(2, p.Unplanned.Count); }
    [Fact] public void ExistingAllocationsNotDuplicated() { var t = Task(); Assert.Empty(Scheduler.Plan([t], [new() { TaskId = t.Id, Start = Day, End = Day.AddMinutes(30) }], Window()).Blocks); }
    [Fact] public void BreaksHaveNoOverlap() { var p = Scheduler.Plan([Task(), Task()], [], Window() with { BreakMinutes = 5 }); Assert.Contains(p.Blocks, b => b.Kind == BlockKind.Break); for (int i = 1; i < p.Blocks.Count; i++) Assert.True(p.Blocks[i].Start >= p.Blocks[i - 1].End); }
    [Fact] public void CompletedTasksExcluded() { var t = Task(); t.Status = Status.Completed; Assert.Empty(Scheduler.Plan([t], [], Window()).Blocks); }
    [Fact] public void DueDateInfluencesOrdering() { var a = Task(); a.Due = Day.AddMinutes(40); var b = Task(); b.Due = Day.AddDays(30); Assert.Equal(a.Id, Scheduler.Plan([b, a], [], Window()).Blocks[0].TaskId); }
    [Fact] public void DuplicateCompletionHasNoSecondRecurrence() { var t = Task(); t.Repeat = Repeat.Daily; t.Due = Day; Assert.NotNull(t.Complete(Day)); Assert.Null(t.Complete(Day)); }
    [Theory][InlineData(Repeat.Daily)][InlineData(Repeat.Weekly)][InlineData(Repeat.Monthly)][InlineData(Repeat.Yearly)] public void RecurrenceMovesIntoFuture(Repeat repeat) { var t = Task(); t.Repeat = repeat; t.Due = Day.AddYears(-2); Assert.True(t.Complete(Day)!.Due > Day); }
    [Fact] public void ReopenedTaskDoesNotDuplicateNextOccurrence() { var t = Task(); t.Repeat = Repeat.Daily; Assert.NotNull(t.Complete(Day)); t.Status = Status.Todo; Assert.Null(t.Complete(Day)); }
    [Fact] public void CustomRecurrence() { var t = Task(); t.Repeat = Repeat.CustomDays; t.RepeatIntervalDays = 3; t.Due = Day; Assert.Equal(Day.AddDays(3), t.Complete(Day)!.Due); }
    [Fact] public void FutureTaskDoesNotLeaveUsableGap() { var future = Task(); future.Priority = Priority.Urgent; future.NotBefore = Day.AddHours(1); var now = Task(); now.Priority = Priority.Low; Assert.Equal(now.Id, Scheduler.Plan([future, now], [], Window()).Blocks[0].TaskId); }
    [Fact] public void OverlappingAvailabilityRejectedEvenWhenBusy() { Assert.Throws<ArgumentException>(() => Scheduler.FreeWindows([new(Day, Day.AddHours(2)), new(Day.AddHours(1), Day.AddHours(3))], [new() { Start = Day, End = Day.AddHours(4) }])); }
    [Fact] public void ImpossibleSplitDoesNotBlockOtherTasks() { var a = Task(minutes: 30); a.Priority = Priority.Urgent; a.MinimumMinutes = 20; var b = Task(minutes: 15); b.Priority = Priority.Low; Assert.Equal(b.Id, Scheduler.Plan([a, b], [], Window(25)).Blocks[0].TaskId); }
    [Fact] public void NamespacedReleaseVersion() { Assert.True(UpdateService.IsNewer("0.1.0-beta.1", "flowstate-v1.0.0")); }
    [Fact] public void ValidationRejectsEmptyTitle() { Assert.Throws<ArgumentException>(() => new TaskItem().Validate()); }
    [Fact] public void CaptureReadsGermanDateAndDuration() { var t = Capture.Parse("Netzwerk lernen am Freitag für 45 Minuten", Day); Assert.Equal("Netzwerk lernen", t.Title); Assert.Equal(45, t.Minutes); Assert.Equal(DayOfWeek.Friday, t.Due!.Value.DayOfWeek); }
    [Fact] public void PausedTimeExcluded() { var t = new FocusClock(); t.Start(Day); t.Pause(Day.AddSeconds(100)); t.Start(Day.AddSeconds(300)); Assert.Equal(200, t.Elapsed(Day.AddSeconds(400))); }
    [Fact] public void ClockCannotGoNegative() { var t = new FocusClock(); t.Start(Day); Assert.Equal(0, t.Elapsed(Day.AddSeconds(-10))); }
    [Fact] public void SleepCountsAsElapsed() { var t = new FocusClock { TargetSeconds = 60 }; t.Start(Day); Assert.True(t.Finished(Day.AddHours(1))); }
    [Fact] public void DstInvalidTimeRejected() { var zone = TimeZoneInfo.FindSystemTimeZoneById("Europe/Zurich"); Assert.Throws<ArgumentException>(() => Scheduler.LocalWindow(new(2026, 3, 29), new(2, 30), new(4, 0), zone)); }
    [Fact] public void DstAmbiguityRejected() { var zone = TimeZoneInfo.FindSystemTimeZoneById("Europe/Zurich"); Assert.Throws<ArgumentException>(() => Scheduler.LocalWindow(new(2026, 10, 25), new(2, 30), new(4, 0), zone)); }
    [Theory][InlineData("1.9.0", "v1.10.0", true)][InlineData("1.0.0", "1.0.0", false)][InlineData("1.0.0", "2.0.0-beta.1", false)][InlineData("invalid", "1.0.0", false)] public void SemanticVersions(string a, string b, bool result) => Assert.Equal(result, UpdateService.IsNewer(a, b));
    [Theory][InlineData("https://github.com/747elias/flowwstate/releases/download/v1.0.0/setup.exe", true)][InlineData("https://evil.example/747elias/flowwstate/releases/download/v1.0.0/setup.exe", false)][InlineData("https://github.com/other/repo/releases/download/v1.0.0/setup.exe", false)][InlineData("http://github.com/747elias/flowwstate/releases/download/v1.0.0/setup.exe", false)] public void DownloadOrigin(string url, bool expected) => Assert.Equal(expected, UpdateService.TrustedAsset(url, "v1.0.0"));
}
