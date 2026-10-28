using NAudio.Wave;
using NAudio.Wave.SampleProviders;
namespace Flowstate.App;

public sealed class AudioMixer : ISampleProvider, IDisposable
{
    readonly Random random = new(); readonly object gate = new();
    WaveOutEvent? output; AudioFileReader? file; ISampleProvider? fileSource;
    float brown, b0, b1, b2, b3, b4, b5, b6; double gain;
    public double White { get; set; }
    public double Brown { get; set; }
    public double Pink { get; set; }
    public double Master { get; set; } = .5;
    public double FileVolume { get; set; } = .4;
    bool playing;
    public WaveFormat WaveFormat { get; } = WaveFormat.CreateIeeeFloatWaveFormat(44100, 2);
    public void Play() { lock (gate) { if (output == null) { output = new WaveOutEvent(); output.Init(this); } playing = true; output.Play(); } }
    public void Pause() { playing = false; }
    public void Load(string path)
    {
        if (!File.Exists(path)) throw new FileNotFoundException("Audiodatei nicht gefunden.");
        if (!new[] { ".wav", ".mp3", ".flac" }.Contains(Path.GetExtension(path).ToLowerInvariant())) throw new ArgumentException("Unterstützt werden WAV, MP3 und FLAC.");
        lock (gate)
        {
            var next = new AudioFileReader(path); ISampleProvider source = next;
            if (source.WaveFormat.Channels == 1) source = new MonoToStereoSampleProvider(source);
            if (source.WaveFormat.Channels != 2) { next.Dispose(); throw new ArgumentException("Bitte Mono- oder Stereo-Audio verwenden."); }
            if (source.WaveFormat.SampleRate != 44100) source = new WdlResamplingSampleProvider(source, 44100);
            file?.Dispose(); file = next; fileSource = source;
        }
    }
    public int Read(float[] buffer, int offset, int count)
    {
        lock (gate)
        {
            Array.Clear(buffer, offset, count);
            if (fileSource != null && playing)
            {
                var read = fileSource.Read(buffer, offset, count); if (read < count && file != null) { file.Position = 0; fileSource.Read(buffer, offset + read, count - read); }
            }
            for (int i = 0; i < count; i += 2)
            {
                float white = (float)(random.NextDouble() * 2 - 1); brown = (brown + .02f * white) / 1.02f;
                b0 = .99886f * b0 + white * .0555179f; b1 = .99332f * b1 + white * .0750759f; b2 = .969f * b2 + white * .153852f; b3 = .8665f * b3 + white * .3104856f; b4 = .55f * b4 + white * .5329522f; b5 = -.7616f * b5 - white * .016898f;
                float pink = (b0 + b1 + b2 + b3 + b4 + b5 + b6 + white * .5362f) * .11f; b6 = white * .115926f;
                gain += ((playing ? Master : 0) - gain) * .00015;
                var noise = (white * White * .18 + brown * Brown * 1.5 + pink * Pink * .4);
                for (int ch = 0; ch < 2 && i + ch < count; ch++) buffer[offset + i + ch] = (float)Math.Tanh((noise + buffer[offset + i + ch] * FileVolume) * gain);
            }
            return count;
        }
    }
    public void Dispose() { output?.Dispose(); file?.Dispose(); }
}
