using System.Text.RegularExpressions;
namespace Flowstate.Core;

public static partial class Capture
{
    [GeneratedRegex(@"(?:für\s+)?(\d{1,4})\s*(?:minuten|min|m)\b", RegexOptions.IgnoreCase)] private static partial Regex Duration();
    public static TaskItem Parse(string input, DateTimeOffset now)
    {
        var match = Duration().Match(input); int minutes = 25;
        if (match.Success) { minutes = int.Parse(match.Groups[1].Value); input = Duration().Replace(input, ""); }
        DateTimeOffset? due = null;
        if (Regex.IsMatch(input, @"\bmorgen\b", RegexOptions.IgnoreCase)) { due = new DateTimeOffset(now.Date.AddDays(1).AddHours(18), now.Offset); input = Regex.Replace(input, @"\bmorgen\b", "", RegexOptions.IgnoreCase); }
        else if (Regex.IsMatch(input, @"\bheute\b", RegexOptions.IgnoreCase)) { due = new DateTimeOffset(now.Date.AddHours(18), now.Offset); input = Regex.Replace(input, @"\bheute\b", "", RegexOptions.IgnoreCase); }
        string[] days = ["Sonntag", "Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag"];
        for (int i = 0; i < days.Length && due == null; i++) if (Regex.IsMatch(input, $@"\b{days[i]}\b", RegexOptions.IgnoreCase))
        { var delta = (i - (int)now.DayOfWeek + 7) % 7; due = new DateTimeOffset(now.Date.AddDays(delta == 0 ? 7 : delta).AddHours(18), now.Offset); input = Regex.Replace(input, $@"(?:am\s+)?\b{days[i]}\b", "", RegexOptions.IgnoreCase); }
        var t = new TaskItem { Title = Regex.Replace(input.Trim(), @"\s+", " "), Minutes = minutes, MinimumMinutes = Math.Min(15, minutes), Due = due }; t.Validate(); return t;
    }
}
