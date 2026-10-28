using Flowstate.Core;
using System.Diagnostics;
var day=new DateTimeOffset(2026,11,2,9,0,0,TimeSpan.Zero);
var windows=Enumerable.Range(0,14).Select(i=>new TimeWindow(day.AddDays(i),day.AddDays(i).AddHours(8))).ToList();
foreach(var count in new[]{100,1000})
{
    var tasks=Enumerable.Range(0,count).Select(i=>new TaskItem{Title="Benchmark",Minutes=15+i%8*15,MinimumMinutes=15,Priority=(Priority)(i%4),Due=day.AddDays(i%14)}).ToList();
    var options=new PlanOptions(windows,BreakMinutes:5);Scheduler.Plan(tasks,[],options);
    var samples=new List<double>();PlanResult last=null;
    for(int i=0;i<10;i++){var clock=Stopwatch.StartNew();last=Scheduler.Plan(tasks,[],options);clock.Stop();samples.Add(clock.Elapsed.TotalMilliseconds);}
    samples.Sort();Console.WriteLine($"{count} tasks, 14 days: median {samples[5]:F2} ms; min {samples[0]:F2}; max {samples[^1]:F2}; {last.Blocks.Count} resulting blocks; {last.Unplanned.Count} unplanned");
}
Console.WriteLine(System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription);
Console.WriteLine(System.Runtime.InteropServices.RuntimeInformation.OSDescription);
