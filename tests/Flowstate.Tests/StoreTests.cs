using Flowstate.Core;
using Flowstate.Infrastructure;
namespace Flowstate.Tests;
public class StoreTests:IDisposable
{
    readonly string path=Path.Combine(Path.GetTempPath(),"flowstate-tests-"+Guid.NewGuid());
    [Fact] public async Task CrudAndRelationshipsSurviveRestart()
    {
        var store=new Store(path);var data=await store.LoadAsync();var p=new Project{Name="Research"};data.Projects.Add(p);var t=new TaskItem{Title="Read",ProjectId=p.Id,Tags=["test"],Checklist=["one"]};data.Tasks.Add(t);data.Blocks.Add(new(){TaskId=t.Id,Start=DateTimeOffset.UtcNow,End=DateTimeOffset.UtcNow.AddMinutes(25)});data.Sessions.Add(new(){TaskId=t.Id,Start=DateTimeOffset.UtcNow,End=DateTimeOffset.UtcNow.AddMinutes(20),Minutes=20});await store.SaveAsync(data);
        var loaded=await new Store(path).LoadAsync();Assert.Equal("test",loaded.Tasks.Single().Tags.Single());Assert.Equal(p.Id,loaded.Tasks.Single().ProjectId);Assert.Single(loaded.Blocks);Assert.Equal(20,loaded.Sessions.Single().Minutes);
        loaded.Tasks[0].Title="Edited";await store.SaveAsync(loaded);Assert.Equal("Edited",(await store.LoadAsync()).Tasks[0].Title);
    }
    [Fact] public async Task BackupRestoreRoundTrip()
    {
        var store=new Store(path);var data=await store.LoadAsync();data.Tasks.Add(new(){Title="Keep"});await store.SaveAsync(data);var backup=await store.BackupAsync(data);data.Tasks.Clear();await store.SaveAsync(data);var restored=await store.ImportAsync(backup,data);Assert.Equal("Keep",restored.Tasks.Single().Title);Assert.Equal(2,Directory.GetFiles(Path.Combine(path,"Backups")).Length);
    }
    [Fact] public async Task InvalidImportDoesNotDestroyData()
    {
        var store=new Store(path);var data=await store.LoadAsync();data.Tasks.Add(new(){Title="Keep"});await store.SaveAsync(data);var file=Path.Combine(path,"bad.json");await File.WriteAllTextAsync(file,"{\"SchemaVersion\":999}");await Assert.ThrowsAsync<InvalidDataException>(()=>store.ImportAsync(file,data));Assert.Equal("Keep",(await store.LoadAsync()).Tasks.Single().Title);
    }
    [Fact] public async Task TimerPersists()
    {
        var store=new Store(path);var data=await store.LoadAsync();data.ActiveTimer=new(){TargetSeconds=60};data.ActiveTimer.Start(DateTimeOffset.UtcNow);await store.SaveAsync(data);Assert.True((await store.LoadAsync()).ActiveTimer!.Running);
    }
    public void Dispose(){Microsoft.Data.Sqlite.SqliteConnection.ClearAllPools();if(Directory.Exists(path))Directory.Delete(path,true);}
}
