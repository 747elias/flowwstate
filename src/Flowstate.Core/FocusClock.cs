namespace Flowstate.Core;
// UTC intervals survive navigation, lock and restart. Sleep counts as elapsed time;
// each interval is clamped at zero after a backwards system-clock adjustment.
public class FocusClock
{
    public Guid SessionId { get; set; } = Guid.NewGuid();
    public Guid? TaskId { get; set; }
    public DateTimeOffset Created { get; set; }
    public DateTimeOffset? RunningSince { get; set; }
    public double AccumulatedSeconds { get; set; }
    public int TargetSeconds { get; set; } = 1500;
    public bool IsBreak { get; set; }
    public bool Stopwatch { get; set; }
    public bool Running => RunningSince != null;
    public double Elapsed(DateTimeOffset now) => AccumulatedSeconds + (RunningSince is {} start ? Math.Max(0, (now - start).TotalSeconds) : 0);
    public double Remaining(DateTimeOffset now) => Math.Max(0, TargetSeconds - Elapsed(now));
    public void Start(DateTimeOffset now) { if (!Running) RunningSince = now; if (Created == default) Created = now; }
    public void Pause(DateTimeOffset now) { AccumulatedSeconds = Elapsed(now); RunningSince = null; }
    public bool Finished(DateTimeOffset now) => !Stopwatch && Elapsed(now) >= TargetSeconds;
}
