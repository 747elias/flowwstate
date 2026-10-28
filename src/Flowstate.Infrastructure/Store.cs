using System.Text.Json;
using Flowstate.Core;
using Microsoft.EntityFrameworkCore;
namespace Flowstate.Infrastructure;

public sealed class Store(string directory)
{
    public string DirectoryPath { get; } = directory;
    public string DatabasePath => Path.Combine(DirectoryPath, "flowstate.db");
    readonly SemaphoreSlim gate = new(1);
    public static JsonSerializerOptions Json { get; } = new() { WriteIndented = true };
    public async Task<AppData> LoadAsync()
    {
        await gate.WaitAsync();
        try
        {
            Directory.CreateDirectory(DirectoryPath);
            await using var db = new Database(DatabasePath);
            var pending = await db.Database.GetPendingMigrationsAsync();
            if (pending.Any())
            {
                if (File.Exists(DatabasePath) && new FileInfo(DatabasePath).Length > 0)
                {
                    var folder = Path.Combine(DirectoryPath, "Backups"); Directory.CreateDirectory(folder);
                    using var source = new Microsoft.Data.Sqlite.SqliteConnection($"Data Source={DatabasePath}");
                    using var target = new Microsoft.Data.Sqlite.SqliteConnection($"Data Source={Path.Combine(folder, $"before-migration-{DateTime.UtcNow:yyyyMMddHHmmss}-{Guid.NewGuid():N}.db")}");
                    source.Open(); target.Open(); source.BackupDatabase(target);
                }
                await db.Database.MigrateAsync();
            }
            var row = await db.Settings.AsNoTracking().SingleOrDefaultAsync();
            var data = row == null ? new AppData() : JsonSerializer.Deserialize<AppData>(row.Json, Json) ?? throw new InvalidDataException("Einstellungen konnten nicht gelesen werden.");
            if (data.SchemaVersion != 1) throw new InvalidDataException("Unbekannte Datenbankversion. Bitte eine passende App-Version verwenden.");
            data.Tasks = await db.Tasks.AsNoTracking().ToListAsync(); data.Projects = await db.Projects.AsNoTracking().ToListAsync();
            data.Blocks = await db.Blocks.AsNoTracking().ToListAsync(); data.Sessions = await db.Sessions.AsNoTracking().ToListAsync();
            data.Workspaces = await db.Workspaces.AsNoTracking().ToListAsync(); data.Reviews = await db.Reviews.AsNoTracking().ToListAsync();
            return data;
        }
        finally { gate.Release(); }
    }
    public static void Validate(AppData d)
    {
        if (d.SchemaVersion != 1) throw new InvalidDataException("Nicht unterstützte Exportversion.");
        foreach (var t in d.Tasks) t.Validate();
        void Unique(IEnumerable<Guid> ids) { var a = ids.ToArray(); if (a.Distinct().Count() != a.Length) throw new InvalidDataException("Doppelte IDs im Import."); }
        Unique(d.Tasks.Select(t => t.Id)); Unique(d.Projects.Select(t => t.Id)); Unique(d.Blocks.Select(t => t.Id)); Unique(d.Sessions.Select(t => t.Id)); Unique(d.Workspaces.Select(t => t.Id)); Unique(d.Reviews.Select(t => t.Id));
        var taskIds = d.Tasks.Select(t => t.Id).ToHashSet(); var projectIds = d.Projects.Select(t => t.Id).ToHashSet();
        if (d.Tasks.Any(t => t.ProjectId.HasValue && !projectIds.Contains(t.ProjectId.Value)) || d.Tasks.Any(t => t.Dependencies.Any(id => !taskIds.Contains(id)))) throw new InvalidDataException("Ungültige Projekt- oder Aufgabenverknüpfung.");
        if (d.Blocks.Any(b => b.End <= b.Start || b.TaskId.HasValue && !taskIds.Contains(b.TaskId.Value)) || d.Sessions.Any(s => s.Minutes < 0 || s.TaskId.HasValue && !taskIds.Contains(s.TaskId.Value))) throw new InvalidDataException("Ungültiger Kalender- oder Fokuseintrag.");
        if (d.Settings.WorkStart is < 0 or > 23 || d.Settings.WorkEnd <= d.Settings.WorkStart || d.Settings.WorkEnd > 24 || d.Settings.FocusMinutes is < 1 or > 1440 || d.Settings.ShortBreak < 1 || d.Settings.LongBreak < 1 || d.Settings.SessionsBeforeLongBreak < 1) throw new InvalidDataException("Ungültige Einstellungen.");
    }
    public async Task SaveAsync(AppData data)
    {
        Validate(data);
        // Freeze before awaiting: a UI edit cannot mutate the in-flight transaction.
        var frozen = JsonSerializer.Deserialize<AppData>(JsonSerializer.Serialize(data, Json), Json)!;
        await gate.WaitAsync();
        try
        {
            await using var db = new Database(DatabasePath);
            await using var tx = await db.Database.BeginTransactionAsync();
            await db.Blocks.ExecuteDeleteAsync(); await db.Sessions.ExecuteDeleteAsync(); await db.Tasks.ExecuteDeleteAsync(); await db.Projects.ExecuteDeleteAsync();
            await db.Workspaces.ExecuteDeleteAsync(); await db.Reviews.ExecuteDeleteAsync(); await db.Settings.ExecuteDeleteAsync();
            db.Projects.AddRange(frozen.Projects); db.Tasks.AddRange(frozen.Tasks); db.Blocks.AddRange(frozen.Blocks); db.Sessions.AddRange(frozen.Sessions);
            db.Workspaces.AddRange(frozen.Workspaces); db.Reviews.AddRange(frozen.Reviews);
            var settings = new AppData { Settings = frozen.Settings, ActiveTimer = frozen.ActiveTimer, PlanUndo = frozen.PlanUndo };
            db.Settings.Add(new() { Json = JsonSerializer.Serialize(settings, Json) });
            await db.SaveChangesAsync(); await tx.CommitAsync();
        }
        finally { gate.Release(); }
    }
    public async Task<string> BackupAsync(AppData data)
    {
        var folder = Path.Combine(DirectoryPath, "Backups"); Directory.CreateDirectory(folder);
        var path = Path.Combine(folder, $"Flowstate-{DateTime.Now:yyyyMMdd-HHmmss}-{Guid.NewGuid():N}.json");
        await ExportAsync(data, path); return path;
    }
    public static async Task ExportAsync(AppData data, string path)
    {
        Validate(data); var temp = path + ".tmp";
        await File.WriteAllTextAsync(temp, JsonSerializer.Serialize(data, Json)); File.Move(temp, path, true);
    }
    public async Task<AppData> ImportAsync(string path, AppData current)
    {
        if (new FileInfo(path).Length > 50_000_000) throw new InvalidDataException("Import ist größer als 50 MB.");
        var data = JsonSerializer.Deserialize<AppData>(await File.ReadAllTextAsync(path), Json) ?? throw new InvalidDataException("Leere Datei.");
        Validate(data); data.ActiveTimer = null; // Never execute imported timers or workspaces.
        await BackupAsync(current); await SaveAsync(data); return data;
    }
}
