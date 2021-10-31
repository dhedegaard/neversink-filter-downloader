# NeverSink filter downloader

[![Rust](https://github.com/dhedegaard/neversink-filter-downloader/actions/workflows/rust-build.yml/badge.svg)](https://github.com/dhedegaard/neversink-filter-downloader/actions/workflows/rust-build.yml)

A small [Rust](https://www.rust-lang.org/) application for fetching and updating to the latest release of the excellent [NeverSink loot filter](https://github.com/NeverSinkDev/NeverSink-Filter).

Already compiled executables can be found under [releases](https://github.com/dhedegaard/neversink-filter-downloader/releases).

### Missing VCRUNTIME140.dll

The executables are compiled with the [Visual C++ Build Tools 2015](http://landinghub.visualstudio.com/visual-cpp-build-tools), these require the Visual C++ redistibutable to run, download it here:
<https://www.microsoft.com/en-us/download/details.aspx?id=48145>

Download and install **vc_redist.x64.exe** and then run the executable again.
