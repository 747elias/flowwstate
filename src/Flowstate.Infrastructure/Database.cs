using Microsoft.EntityFrameworkCore;
using Flowstate.Core;
namespace Flowstate.Infrastructure;

public class SettingRow { public int Id { get; set; } = 1; public string Json { get; set; } = "{}"; }
public class Database(string path) : DbContext
{
    public DbSet<TaskItem> Tasks => Set<TaskItem>();
    public DbSet<Project> Projects => Set<Project>();
    public DbSet<CalendarBlock> Blocks => Set<CalendarBlock>();
    public DbSet<FocusSession> Sessions => Set<FocusSession>();
    public DbSet<Workspace> Workspaces => Set<Workspace>();
    public DbSet<DailyReview> Reviews => Set<DailyReview>();
    public DbSet<SettingRow> Settings => Set<SettingRow>();
    protected override void OnConfiguring(DbContextOptionsBuilder builder) => builder.UseSqlite($"Data Source={path}");
    protected override void OnModelCreating(ModelBuilder b)
    {
        b.Entity<TaskItem>().HasIndex(t => t.Status);
        b.Entity<TaskItem>().HasIndex(t => t.ProjectId);
        b.Entity<TaskItem>().HasOne<Project>().WithMany().HasForeignKey(t => t.ProjectId).OnDelete(DeleteBehavior.SetNull);
        b.Entity<CalendarBlock>().HasOne<TaskItem>().WithMany().HasForeignKey(t => t.TaskId).OnDelete(DeleteBehavior.SetNull);
        b.Entity<FocusSession>().HasOne<TaskItem>().WithMany().HasForeignKey(t => t.TaskId).OnDelete(DeleteBehavior.SetNull);
        b.Entity<CalendarBlock>().HasIndex(t => t.Start);
        b.Entity<FocusSession>().HasIndex(t => t.Start);
        b.Entity<DailyReview>().HasIndex(t => t.Date).IsUnique();
    }
}
