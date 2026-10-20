"""Taurino Build CLI
Ich würde für Taurino eine eigene Build-CLI entwickeln, die ähnlich wie tauri build funktioniert und sowohl PyInstaller als auch Nuitka unterstützt.
Die Benutzer sollen ihre Python-Anwendung direkt über Taurino bauen können, ohne die einzelnen Compilerbefehle selbst schreiben zu müssen.
1. Gewünschte Befehle
Standard-Build
PyInstaller
taurino build main.py
Nuitka-Build
taurino build main.py --backend nuitka
Eine ausführbare Datei
taurino build main.py --onefile
GUI ohne Konsolenfenster
taurino build main.py --backend nuitka --onefile --windowed
Benutzerdefinierter Build
taurino build main.py --backend nuitka --onefile --name MyApp --output-dir dist
Die CLI soll das installierte Build-Backend verwenden, den Build-Prozess ausführen, Fehlercodes korrekt zurückgeben und deine vorhandenen Packaging-Konfigurationen berücksichtigen.



"""  # noqa: E501

from __future__ import annotations

from .build import BuildConfig, run_build

__all__ = ["BuildConfig", "run_build"]
