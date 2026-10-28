using Microsoft.UI.Xaml;
using Microsoft.Extensions.DependencyInjection;
using Flowstate.Infrastructure;
namespace Flowstate.App;

public partial class App : Application
{
    public static MainWindow? Main { get; private set; }
    public static ServiceProvider Services { get; private set; } = null!;
    static Mutex? instance;
    public App()
    {
        UnhandledException += (_, e) => Log(e.Exception);
        try
        {
            InitializeComponent();
            var directory = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Flowstate");
            Directory.CreateDirectory(directory);
            Services = new ServiceCollection().AddSingleton(new Store(directory)).AddSingleton<UpdateService>().AddSingleton<MainViewModel>().BuildServiceProvider();
        }
        catch(Exception e) { Log(e); throw; }
    }
    protected override void OnLaunched(LaunchActivatedEventArgs args) { instance = new Mutex(true, "Local\\Flowstate-" + Environment.UserName, out bool first); if (!first) { Exit(); return; } ThemeResources.Initialize(Resources); Main = new MainWindow(Services.GetRequiredService<MainViewModel>()); Main.Activate(); }
    public static void Log(Exception e)
    {
        try { var folder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Flowstate", "Logs"); Directory.CreateDirectory(folder); File.AppendAllText(Path.Combine(folder, "errors.log"), $"{DateTimeOffset.UtcNow:O} {e.GetType().Name} HRESULT={e.HResult:X8} {e.Message} {e.StackTrace}\n"); } catch { System.Diagnostics.Debug.WriteLine("Logging failed."); }
    }
}
