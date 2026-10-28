using Microsoft.EntityFrameworkCore.Design;
namespace Flowstate.Infrastructure;

public class DesignFactory : IDesignTimeDbContextFactory<Database>
{
    public Database CreateDbContext(string[] args) => new(Path.Combine(Path.GetTempPath(), "flowstate-design.db"));
}
