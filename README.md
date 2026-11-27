# Vice-Tay — profil 7800X3D

Fork de [Vice-Tay](https://github.com/TaynanGT/Vice-Tay) (couche de compatibilité
[VIctory](https://github.com/Marvisak/VIctory)), basé sur la version `146c9e0`,
réglé et mesuré sur un Ryzen 7 7800X3D + RTX 4080 + 32 Go de RAM.

**Aucun fichier du jeu n'est inclus.** Il faut votre propre copie.

## Ce que ce fork change

- **Profil `-Machine 7800x3d`**, choisi automatiquement avec une carte NVIDIA et
  un processeur de 16 threads ou moins. Le profil d'origine (fait pour un 16 cœurs)
  lançait plus de threads de travail que le CPU n'en a. Mesuré en Series S 1080p :
  **15,5 → 21,9 fps** médian dans le monde, pire 10 % 13,5 → 18,3.
- **`VICTORY_WAVE_WIDE_ONLY` implémenté.** La liste était définie par `play.ps1`
  mais lue nulle part : deux shaders ne se traduisaient pas et leurs draws étaient
  abandonnés (têtes invisibles, visages mal rendus). Un cinquième shader
  (`a10179ab7eda0dc3`) est ajouté à la liste.
- **`scripts\fps-report.ps1`** : résume les fps d'une session à partir du log
  (`-More @{VICTORY_LOG_FILE='...'}`), pour comparer deux réglages.

## Jouer

```powershell
git clone https://github.com/kaultakaulta75-dev/Vice-Tay-7800X3D.git
cd Vice-Tay-7800X3D
copy victory.example.toml victory.toml      # puis indiquez vos chemins, hors de ce dossier
cargo build --release --locked -p victory
scripts\play.ps1 -Exe <votre game_scarlett_master.exe> -Console series-s -Build release
```

Le profil 8 cœurs reprend les réglages du profil `-Machine 4070`
d'[AnyXboxX](https://github.com/LooperSalty/AnyXboxX).

## Limites connues

Reflets de l'eau par blocs, quelques taches de couleur sur les visages, zones
d'ombre trop sombres sans `-RayTracing` (qui coûte environ 5 fps). Ne réduisez pas
`VICTORY_OUTPUT` en Series S : l'upscaler du jeu n'y accepte que 1920×1080.

Licence GPL-3.0-or-later, comme VIctory.

---

*Ci-dessous, le README d'origine de Vice-Tay (en anglais).*

# Vice-Tay

Vice-Tay runs an Xbox Series title natively on Windows, from files you supply,
through [VIctory](https://github.com/Marvisak/VIctory), a compatibility layer:
it loads the console executable, answers its system calls and translates its
GPU work to Direct3D 12. This tree is tuned and fixed on one PC, an RTX 4090
with a Ryzen 9 9950X3D and 48 GB of RAM, toward one goal: the open world at
30 frames a second, with the picture right, fully playable.

**No game files, code or assets are in this repository.** You need your own
copy of the game. Vice-Tay is not affiliated with or endorsed by the game's
developer or publisher.

> [!TIP]
> **[View the gameplay screenshots](https://github.com/TaynanGT/Vice-Tay-Screenshots#readme)**
>
> Ten captures from an earlier Series S playtest, on build `672cf831`.

## At a glance

- **It plays.** On the Series S profile the game goes from the prison into the
  prologue's boat section. The boat is driven by hand; when it runs aground the
  mission fails, and the game repeats or skips the checkpoint.
- **It is not fast enough yet.** The world runs at **21.1 fps** (median;
  19.1-23.2 for most frames), short of the 30 fps goal.
- **Memory levels off.** 23.5 GB at the median and 24.1 GB at most over 30
  minutes in the world, about 1 GB more in the last ten minutes than in the
  first; the frame rate eased from 21.7 to 20.6 fps over the same span.

## Test PC

| Component | Configuration |
| --- | --- |
| CPU | AMD Ryzen 9 9950X3D, 16 cores and 32 threads |
| Motherboard | MSI MAG X870 TOMAHAWK WIFI (MS-7E51), revision 1.0 |
| BIOS | MSI 1.A93 |
| GPU | NVIDIA GeForce RTX 4090, 24 GB GDDR6X |
| NVIDIA driver | 621.40 |
| Integrated graphics | AMD Radeon Graphics |
| RAM | 48 GiB DDR5 (2 × 24 GiB), configured at 8,000 MT/s; 43.4 GiB visible to Windows |
| Page file | 128 GiB |
| System SSD | Samsung 990 PRO 4 TB NVMe |
| Additional SSDs | Samsung 990 PRO 2 TB NVMe and Samsung 970 PRO 1 TB NVMe |
| Monitor | Samsung Odyssey OLED G8 (G81SF), 3840 × 2160, 240 Hz |
| OS | Windows 11 Pro 26H2 x64, build 26300.10075 |

## Measurements

<!-- measurements:start -->
### Measured session (build bc090416)

On the test PC above, with `scripts\play.ps1 -Console series-s` and its RTX 4090
settings, in full screen, played by hand: the prologue from the prison into the
boat section, the boat driven to the package and along the marked route to an
island, three failed attempts picked up again from a checkpoint (one of them
skipped), and a capture every minute. Nothing compiled meanwhile; the screen was
recorded to video on the card's encoder throughout.

| Profile | In the world | To the start screen / to the world | fps: median (p10-p90; 1% low) | ms per frame (median) | Game memory, median / max | Card: memory max, load, power, temperature |
|---|---|---|---|---|---|---|
| Series S | 30 min, ended by the runner | 72 s / 120 s | **21.1** (19.1-23.2; 1% low 17.0) | 47.3: 33.7 preparing, 12.6 copying back | 23.5 / 24.1 GB | 8.8 GB, 86.0%, 134.5 W, 49.6 C |

Every number is from the game's own once-a-second frame lines in the world
(over 1,000 draws a frame) and samples every 20 s; the full tables, with each
part of a frame, are in [docs/measurements.md](docs/measurements.md).
<!-- measurements:end -->

What these runs found and fixed on this PC is in
[docs/findings-rtx4090.md](docs/findings-rtx4090.md).

## Known problems

- **Frame rate.** Below 30 fps in the world. Most of a frame goes to preparing the
  GPU's work on the CPU and copying data back, not to the GPU itself.
- **Memory.** On a PC with less free RAM than the game holds (above), Windows
  pages part of it out and frames stall.
- **Picture.** Skin, hair and some walls show flat bands of colour; some objects
  (tanks, tarps, plants, a skylight) glow in flat purple, pink or blue; some faces
  show scrambled colour patches and some eyes are lit white; clouds and reflections
  break into square blocks; a few shapes render as flat black.
  For banding that varies between launches, pass `-More @{VICTORY_RTV_FORMAT_LOG='1'}`
  to `scripts\play.ps1` on a diagnostics build. Compare the `RTV format check`
  totals and `RTV format mismatch` examples at the same checkpoint in a clean and a banded run.
- **Ray tracing.** The ray-traced structures are not built by default, so a few
  hundred dispatches a frame are skipped and some light is missing; `-RayTracing`
  builds them.
- **Loading.** The first run of a new build translates every shader while it plays;
  translate them ahead (below).
- **Measuring.** On the Series S profile the log's GPU-time counter reads near zero,
  so the README's table leaves it out until it is clear what it measures there.

## Getting started

You need Windows 10 or 11 x64, a CPU with AVX2, a Direct3D 12 GPU with shader
model 6.6 and mesh shaders, Rust (the toolchain is pinned in the repository),
the MSVC build tools with a Windows SDK, and your own dump of the title.

```powershell
git clone https://github.com/TaynanGT/Vice-Tay
cd Vice-Tay
copy victory.example.toml victory.toml      # then set your paths, outside this folder
cargo build --locked --profile diag -p victory --features diagnostics
```

From the repository root, translate the shaders and warm the pipelines (each can
be stopped and resumed). With `game_dir` set in `victory.toml`, the shader
command scans that directory. To scan one archive, pass its quoted path, for
example `.\target\diag\victory.exe shaders "D:\My Game Dump\shaders.rpf"`.

```powershell
.\target\diag\victory.exe shaders       # scans game_dir in victory.toml
.\target\diag\victory.exe warm --used   # pipelines earlier runs drew with
```

Then play on the Series S profile, the one measured above:

```powershell
scripts\play.ps1 -Exe <path of your copy's executable> -Console series-s
```

`Play-VIctory.cmd` takes the same options from a Command Prompt; on a double-click
PowerShell asks for the executable's path.

| Option | What it does |
|---|---|
| (none) | Full screen, Series X profile, settings picked for your graphics card, performance panel on |
| **F11** or **Alt+Enter** | Switch between full screen and a window while it runs |
| **F3** | Hide or show the performance panel (FPS, frame times, GPU, CPU, VRAM and RAM) |
| `-NoPanel` | Start without the performance panel |
| `-Windowed` | Start in a window |
| `-Console series-s` | Run as a Series S: a smaller picture, lighter on the card, faster; the profile measured above |
| `-Machine 4090` / `7900xtx` | Force the settings for NVIDIA (measured on the RTX 4090) or AMD |
| `-Safe` | Leave out the changes still on trial; try it first when something goes wrong |
| `-RayTracing` | Build the ray-traced structures, so the title's ray-traced indirect light runs |
| `-Build release` | Run `target\release\victory.exe` (`cargo build --release --locked -p victory`) |

Every `VICTORY_...` switch is described in a comment where the code reads it.
More commands and debugging switches:
[docs/README-fxceless.md](docs/README-fxceless.md) and [docs/findings.md](docs/findings.md);
recording one frame and running it again: [docs/replay.md](docs/replay.md).

## How this tree is built

Vice-Tay is published as a repository of its own, not as a GitHub fork, with the
whole history of the trees it comes from. It started from codec-d/cool_fox (now
taken down), which is [avatarmanz/VIctory-7900XTX](https://github.com/avatarmanz/VIctory-7900XTX)
with work for NVIDIA cards on top. New work from avatarmanz is merged as it lands,
and our own fixes come in through pull requests, each tested before it is merged.
What each change does: [docs/this-fork.md](docs/this-fork.md). How the layer
works: [docs/architecture.md](docs/architecture.md).

## Credits

This is other people's work with more on top. The history is kept, and commits
taken from elsewhere name the original commit and its author. Everyone with a
commit here is listed in [docs/credits.md](docs/credits.md).

- **[VIctory](https://github.com/Marvisak/VIctory)** by Marvisak and its
  contributors: the loader, the runtime services, the Direct3D 12 back end and
  the shader recompiler.
- **[fxceless/victory-fork](https://github.com/fxceless/victory-fork)** and the
  people whose pull requests it carries.
- **[avatarmanz/VIctory-7900XTX](https://github.com/avatarmanz/VIctory-7900XTX)**:
  list calls recorded on the title's threads and translated by workers,
  submissions prepared on threads of their own, cheaper write guards, textures on
  a copy queue, frame record and replay.
- **codec-d/cool_fox** (taken down by its author): two-plane composition, fallback
  heaps, table plans, the residency guard, fence wakes on `WaitOnAddress`, wave32
  lowering for NVIDIA, movie-plane uploads and XMA audio fixes.
- **[DexNom/VIctory](https://github.com/DexNom/VIctory)**: ray tracing fixes,
  cube gradients, sparse textures.
- **[Belobug/VIctory-Belobug-AMD](https://github.com/Belobug/VIctory-Belobug-AMD)**:
  the ambient light idea, pipeline workers, and work by romainhedouin.
- **[FQY55kai/ZJOU-VICTORY](https://github.com/FQY55kai/ZJOU-VICTORY)**.
- **[myokex/victory-amd](https://github.com/myokex/victory-amd)**: the guard over
  every view of a page (with Spexivus, joroyt and tightfall), and Joshua Odom's
  reads into title memory.
- **Geckomacabre** and **cxy0072019-sketch**: the texture read-back.
- **[XCL](https://github.com/ivall/xcl)** (GPL-2.0-only) was read for how the same
  title behaves under another layer; no code of it is here.

## License

GPL-3.0-or-later, as VIctory. See [LICENSE](LICENSE); the one file it does not
cover is named in [docs/asset-policy.md](docs/asset-policy.md).
