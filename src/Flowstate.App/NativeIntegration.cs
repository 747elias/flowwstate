using System.Runtime.InteropServices;
namespace Flowstate.App;
public sealed class NativeIntegration : IDisposable
{
    readonly MainWindow window; readonly nint hwnd; readonly Action capture;readonly Action focus;
    readonly SubclassProc callback;NotifyData icon;bool registered;
    const uint Callback=0x8001;const int HotkeyId=70;
    public NativeIntegration(MainWindow window,Action capture,Action focus)
    {
        this.window=window;this.capture=capture;this.focus=focus;hwnd=WinRT.Interop.WindowNative.GetWindowHandle(window);callback=WndProc;
        if(!SetWindowSubclass(hwnd,callback,1,0))throw new InvalidOperationException("Windows-Fensterintegration konnte nicht initialisiert werden.");
        icon=new NotifyData { cbSize=(uint)Marshal.SizeOf<NotifyData>(),hWnd=hwnd,uID=1,uFlags=7,uCallbackMessage=Callback,hIcon=LoadImage(0,Path.Combine(AppContext.BaseDirectory,"Assets","Flowstate.ico"),1,0,0,0x10|0x40),szTip="Flowstate",szInfo="",szInfoTitle="" };
        if(!Shell_NotifyIcon(0,ref icon))throw new InvalidOperationException("Infobereich konnte nicht initialisiert werden.");
    }
    public void Register(uint key)
    {
        if(registered)UnregisterHotKey(hwnd,HotkeyId);
        registered=RegisterHotKey(hwnd,HotkeyId,0x4000|0x0001|0x0002,key);
        if(!registered)Notify("Tastenkürzel belegt","Ändere das Quick-Capture-Tastenkürzel in den Einstellungen.");
    }
    public void Notify(string title,string message)
    { icon.uFlags=0x10;icon.szInfoTitle=title;icon.szInfo=message;icon.dwInfoFlags=1;Shell_NotifyIcon(1,ref icon); }
    nint WndProc(nint h,uint msg,nint w,nint l,nuint id,nuint data)
    {
        if(msg==0x0312 && w==HotkeyId){capture();return 0;}
        if(msg==Callback)
        {
            if(l==0x0203)window.ShowWindow();
            if(l==0x0205)
            {
                var menu=CreatePopupMenu();AppendMenu(menu,0,1,"Flowstate öffnen");AppendMenu(menu,0,2,"Focus Studio");AppendMenu(menu,0,3,"Timer starten / pausieren / fortsetzen");AppendMenu(menu,0,4,"Aufgabe erfassen");AppendMenu(menu,0,5,"Beenden");
                GetCursorPos(out var p);SetForegroundWindow(hwnd);int selected=TrackPopupMenu(menu,0x100|0x2,p.X,p.Y,0,hwnd,0);DestroyMenu(menu);
                switch(selected){case 1:window.ShowWindow();break;case 2:window.ShowWindow();focus();break;case 3:_=window.ToggleTimerAsync();break;case 4:capture();break;case 5:_=window.ExitAsync();break;}
            }
        }
        return DefSubclassProc(h,msg,w,l);
    }
    public void Dispose(){if(registered)UnregisterHotKey(hwnd,HotkeyId);Shell_NotifyIcon(2,ref icon);RemoveWindowSubclass(hwnd,callback,1);if(icon.hIcon!=0)DestroyIcon(icon.hIcon);}
    delegate nint SubclassProc(nint h,uint m,nint w,nint l,nuint id,nuint data);
    [StructLayout(LayoutKind.Sequential)]struct Point{public int X,Y;}
    [StructLayout(LayoutKind.Sequential,CharSet=CharSet.Unicode)]struct NotifyData
    {public uint cbSize;public nint hWnd;public uint uID,uFlags,uCallbackMessage;public nint hIcon;[MarshalAs(UnmanagedType.ByValTStr,SizeConst=128)]public string szTip;public uint dwState,dwStateMask;[MarshalAs(UnmanagedType.ByValTStr,SizeConst=256)]public string szInfo;public uint uTimeoutOrVersion;[MarshalAs(UnmanagedType.ByValTStr,SizeConst=64)]public string szInfoTitle;public uint dwInfoFlags;public Guid guidItem;public nint hBalloonIcon;}
    [DllImport("comctl32.dll")]static extern bool SetWindowSubclass(nint h,SubclassProc p,nuint id,nuint data);
    [DllImport("comctl32.dll")]static extern bool RemoveWindowSubclass(nint h,SubclassProc p,nuint id);
    [DllImport("comctl32.dll")]static extern nint DefSubclassProc(nint h,uint m,nint w,nint l);
    [DllImport("shell32.dll",CharSet=CharSet.Unicode)]static extern bool Shell_NotifyIcon(uint m,ref NotifyData d);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)]static extern nint LoadImage(nint instance,string name,uint type,int cx,int cy,uint flags);
    [DllImport("user32.dll")]static extern bool DestroyIcon(nint h);
    [DllImport("user32.dll")]static extern bool RegisterHotKey(nint h,int id,uint modifiers,uint key);
    [DllImport("user32.dll")]static extern bool UnregisterHotKey(nint h,int id);
    [DllImport("user32.dll")]static extern nint CreatePopupMenu();
    [DllImport("user32.dll",CharSet=CharSet.Unicode)]static extern bool AppendMenu(nint h,uint flags,nuint id,string text);
    [DllImport("user32.dll")]static extern bool GetCursorPos(out Point point);
    [DllImport("user32.dll")]static extern bool SetForegroundWindow(nint h);
    [DllImport("user32.dll")]static extern int TrackPopupMenu(nint menu,uint flags,int x,int y,int reserved,nint h,nint rect);
    [DllImport("user32.dll")]static extern bool DestroyMenu(nint menu);
}
