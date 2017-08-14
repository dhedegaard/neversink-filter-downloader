# NeverSink filter downloader

A small [Rust](https://www.rust-lang.org/) application for fetching and updating to the latest release of the excellent [NeverSink loot filter](https://github.com/NeverSinkDev/NeverSink-Filter).

Already compiled executables can be found under [releases](https://github.com/dhedegaard/neversink-filter-downloader/releases).

### Missing VCRUNTIME140.dll

The compiled executables are compiled with Visual C++ Build Tools, these require the Visual C++ redistibutable found here:
<https://www.microsoft.com/en-us/download/details.aspx?id=48145>

Download and install the x64 setup file.