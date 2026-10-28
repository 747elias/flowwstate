using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Windows.UI;
namespace Flowstate.App;

public static class ThemeResources
{
    static SolidColorBrush Brush(byte red, byte green, byte blue) => new(Color.FromArgb(255, red, green, blue));
    public static ResourceDictionary Create()
    {
        var resources = new ResourceDictionary();
        resources.MergedDictionaries.Add(new XamlControlsResources());
        var dark = new ResourceDictionary
        {
            ["CardBackground"] = Brush(34, 35, 41),
            ["MutedForeground"] = Brush(166, 167, 178),
            ["CardStroke"] = Brush(52, 53, 62)
        };
        var light = new ResourceDictionary
        {
            ["CardBackground"] = Brush(255, 255, 255),
            ["MutedForeground"] = Brush(98, 99, 113),
            ["CardStroke"] = Brush(228, 228, 236)
        };
        resources.ThemeDictionaries["Dark"] = dark;
        resources.ThemeDictionaries["Light"] = light;
        resources["FlowAccent"] = Brush(139, 128, 255);
        resources["FlowSuccess"] = Brush(74, 203, 165);
        resources["FlowWarning"] = Brush(233, 170, 101);
        return resources;
    }
}
