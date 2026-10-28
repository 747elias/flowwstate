using System.Net.Http.Json;
using System.Security.Cryptography;
using System.Text.Json.Serialization;
using System.Text.RegularExpressions;
using NuGet.Versioning;
namespace Flowstate.Infrastructure;

public record ReleaseAsset([property: JsonPropertyName("name")] string Name, [property: JsonPropertyName("browser_download_url")] string Url);
public record ReleaseInfo([property: JsonPropertyName("tag_name")] string Tag, [property: JsonPropertyName("body")] string Notes, [property: JsonPropertyName("prerelease")] bool Prerelease, [property: JsonPropertyName("draft")] bool Draft, [property: JsonPropertyName("assets")] List<ReleaseAsset> Assets);
public sealed class UpdateService
{
    public const string Repository = "747elias/flowwstate";
    readonly HttpClient client = new() { Timeout = TimeSpan.FromMinutes(10) };
    public UpdateService() { client.DefaultRequestHeaders.UserAgent.ParseAdd("Flowstate/0.1.0"); }
    public static string VersionText(string tag) => tag.StartsWith("flowstate-v", StringComparison.Ordinal) ? tag[11..] : tag.TrimStart('v');
    public static bool IsNewer(string current, string incoming) => NuGetVersion.TryParse(VersionText(current), out var c) && NuGetVersion.TryParse(VersionText(incoming), out var n) && !n.IsPrerelease && n > c;
    public async Task<ReleaseInfo?> CheckAsync(string current, CancellationToken ct = default)
    {
        using var response = await client.GetAsync($"https://api.github.com/repos/{Repository}/releases/latest", ct);
        if (response.StatusCode == System.Net.HttpStatusCode.NotFound) return null;
        response.EnsureSuccessStatusCode();
        var release = await response.Content.ReadFromJsonAsync<ReleaseInfo>(cancellationToken: ct);
        return release is { Draft: false, Prerelease: false } && IsNewer(current, release.Tag) ? release : null;
    }
    public static bool TrustedAsset(string url, string tag) => Uri.TryCreate(url, UriKind.Absolute, out var u) && u.Scheme == "https" && u.Host == "github.com" && u.IsDefaultPort && string.IsNullOrEmpty(u.UserInfo) && string.IsNullOrEmpty(u.Query) && string.IsNullOrEmpty(u.Fragment) && u.AbsolutePath.StartsWith($"/{Repository}/releases/download/{Uri.EscapeDataString(tag)}/", StringComparison.Ordinal);
    public async Task<string> DownloadAsync(ReleaseInfo release, string cache, CancellationToken ct = default)
    {
        if (!NuGetVersion.TryParse(VersionText(release.Tag), out var version) || version.IsPrerelease || release.Draft || release.Prerelease) throw new InvalidDataException("Ungültige stabile Version.");
        var name = $"Flowstate-Setup-{version.ToNormalizedString()}-win-x64.exe";
        var installer = release.Assets.Single(a => a.Name == name); var checksum = release.Assets.Single(a => a.Name == "SHA256SUMS.txt");
        if (!TrustedAsset(installer.Url, release.Tag) || !TrustedAsset(checksum.Url, release.Tag)) throw new InvalidDataException("Unerwartete Downloadquelle.");
        var checksums = await client.GetStringAsync(checksum.Url, ct);
        var line = checksums.Split('\n').Select(l => l.Trim()).Single(l => Regex.IsMatch(l, @"^[a-fA-F0-9]{64}\s+\*?" + Regex.Escape(name) + "$"));
        var expected = line[..64]; Directory.CreateDirectory(cache); var path = Path.Combine(cache, Guid.NewGuid().ToString("N") + ".exe");
        try
        {
            using var response = await client.GetAsync(installer.Url, HttpCompletionOption.ResponseHeadersRead, ct); response.EnsureSuccessStatusCode();
            if (response.Content.Headers.ContentLength > 600_000_000) throw new InvalidDataException("Installer ist unerwartet groß.");
            await using (var output = File.Create(path))
            {
                await using var stream = await response.Content.ReadAsStreamAsync(ct); var buffer = new byte[81920]; long total = 0; int count;
                while ((count = await stream.ReadAsync(buffer, ct)) != 0) { total += count; if (total > 600_000_000) throw new InvalidDataException("Download zu groß."); await output.WriteAsync(buffer.AsMemory(0, count), ct); }
            }
            await using var input = File.OpenRead(path); var hash = Convert.ToHexString(await SHA256.HashDataAsync(input, ct));
            if (!hash.Equals(expected, StringComparison.OrdinalIgnoreCase)) throw new InvalidDataException("SHA-256-Prüfung fehlgeschlagen.");
            return path;
        }
        catch { File.Delete(path); throw; }
    }
}
