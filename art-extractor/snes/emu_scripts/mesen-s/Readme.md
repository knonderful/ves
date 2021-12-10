# LUA script for Mesen-S

This document contains some relevant information about the artifact extraction from games using the [Mesen-S](https://mesen.ca/) emulator.

## Environment

The current version of Mesen-S at the time of this writing is 0.4.0 and is normally run on Ubuntu 20.04 using the following Mono version:

```
Mono JIT compiler version 6.8.0.105 (Debian 6.8.0.105+dfsg-2 Wed Feb 26 23:23:50 UTC 2020)
Copyright (C) 2002-2014 Novell, Inc, Xamarin Inc and Contributors. www.mono-project.com
	TLS:           __thread
	SIGSEGV:       altstack
	Notifications: epoll
	Architecture:  amd64
	Disabled:      none
	Misc:          softdebug
	Interpreter:   yes
	LLVM:          supported, not enabled.
	Suspend:       hybrid
	GC:            sgen (concurrent by default)
```

## Extracting artifacts

Mesen-S supports LUA scripting as a way to interact with a running emulation. In the context of artifact extraction the most sensible
approach is to record a "movie" (not a "video") of some gameplay and use that as the basis for extraction. This keeps the state consistent
between extractions, since a movie is basically a state save that is replayed with recorded inputs.

The provided `sprite_extracto.lua` script can be loaded into Mesen-S and will capture some JSON data for each frame at the *start* of the
frame. It is important that this is done at the start of the frame and not somewhere else, since the SNES may start overwriting VRAM or OAM
data while the frame is being rendered to the screen, resulting in incorrect captures. (In the past this has resulted in occasional
sprite corruption in Yoshi's Island captures, but it can theoretically happen anywhere.)

The current state of `sprite_extracto.lua` tries to extract only the relevant portions of memory, while trying to avoid too much complexity
in the script itself. For sprite extraction (which is the only thing that is supported at the moment) the following is relevant:

* A copy of CGRAM.
  * Actually, only the latter half of the CGRAM is required, since that is the section that is used for OBJs, but we copy the entire table
    for the sake of simplicity and to avoid confusion when reading the data in the Rust code.
* A copy of the OAM.
* A copy of the OBJ NAME BASE data.
    * This table contains the first half of the graphics referenced by the OAM entries. (See sections A-1 through A-4 in the SNES Developer
      Manual.)
* A copy of the OBJ NAME SELECT data.
    * This table contains the second half of the graphics referenced by the OAM entries.
* (TODO: See the "TODO" section below.)

All this data is written in a JSON file (one per frame). Older versions of the script attempted to collect all the data and write it when
the recording is stopped or the script is terminated, which quickly resulted in emulator crashes (probably due to out-of-memory issues). The
data can be read with the `Frame` struct in the `mesen` Rust module inside this crate.

The `sprite_extracto.lua` script has only ever been tested with a Yoshi's Island ROM, but should theoretically also work with other games.
At the same time, it is likely that the script will have to be more intelligent and extract more state information from the emulator to work
correctly in all cases.

### TODO

The current version of `sprite_extracto.lua` does not extract the OBJ SIZE SELECT information (see Chapter 24 in the SNES Developer Manual)
from the emulator. Instead, it presumes that the mode is `0`, which means that "small" OBJs are 8x8 pixels and "large" OBJs are 16x16 pixels
(which seems to be the only mode used in Yoshi's Island). This needs to be extracted and added to the output JSON. Unfortunately, it is not
clear which field from `emu.getState().ppu` provides this information, but the only sensible option would be the `oamMode`, since that is
the only property that "could" be the one that we need (based on the naming) and it also contains the expected value in the sample (namely
`0`). It probably makes sense to have a look at the C++ source code to confirm this somehow.

## Mesen-S LUA support

Mesen-S has support for LUA scripts. However, some of its functionality is either undocumented or not clearly documented. The following
section describes any such points.

### The `emu.getState()` function

The documentation says nothing about what this data structure actually contains. There are two ways to find out more: look into the C++
source code or extract a sample of the data and use that as a reference. The latter approach is used here. The following is a sample output
of a `emu.getState()` call from Yoshi's Island:

```
return
{
	["masterClock"] = -753824024,
	["spc"] = 
	{
		["x"] = 74,
		["a"] = 255,
		["pc"] = 1110,
		["status"] = 11,
		["sp"] = 207,
		["y"] = 3,
	},
	["cpu"] = 
	{
		["x"] = 9,
		["nmiFlag"] = false,
		["irqFlag"] = 0,
		["emulationMode"] = false,
		["sp"] = 500,
		["k"] = 126,
		["cycleCount"] = -571740731,
		["a"] = 32,
		["pc"] = 56928,
		["d"] = 0,
		["db"] = 4,
		["y"] = 61,
		["status"] = 16,
	},
	["ppu"] = 
	{
		["mosaicEnabled"] = 0,
		["frameCount"] = 190185,
		["windowMaskLogicSprites"] = false,
		["windowMaskMainBg1"] = false,
		["windowMaskLogicBg1"] = false,
		["vramAddrIncrementOnSecondReg"] = true,
		["windowMaskLogicBg2"] = false,
		["screenInterlace"] = false,
		["colorMathEnabled"] = 32,
		["colorMathHalveResult"] = false,
		["windowMaskLogicBg3"] = false,
		["windowMaskMainBg0"] = false,
		["colorMathSubstractMode"] = false,
		["colorMathAddSubscreen"] = true,
		["layers"] = 
		{
			[1] = 
			{
				["doubleWidth"] = 0,
				["chrAddress"] = 28672,
				["vScroll"] = 802,
				["hScroll"] = 187,
				["doubleHeight"] = 1,
				["tilemapAddress"] = 14336,
				["largeTiles"] = 1,
			},
			[2] = 
			{
				["doubleWidth"] = 0,
				["chrAddress"] = 8192,
				["vScroll"] = 275,
				["hScroll"] = 448,
				["doubleHeight"] = 0,
				["tilemapAddress"] = 13312,
				["largeTiles"] = 1,
			},
			[3] = 
			{
				["doubleWidth"] = 0,
				["chrAddress"] = 0,
				["vScroll"] = 1023,
				["hScroll"] = 0,
				["doubleHeight"] = 0,
				["tilemapAddress"] = 0,
				["largeTiles"] = 0,
			},
			[0] = 
			{
				["doubleWidth"] = 1,
				["chrAddress"] = 28672,
				["vScroll"] = 764,
				["hScroll"] = 374,
				["doubleHeight"] = 0,
				["tilemapAddress"] = 26624,
				["largeTiles"] = 0,
			},
		},
		["colorMathPreventMode"] = 2,
		["enableOamPriority"] = false,
		["overscanMode"] = false,
		["directColorMode"] = false,
		["windowMaskSubBg0"] = false,
		["fixedColor"] = 19876,
		["windowMaskMainSprites"] = false,
		["scanline"] = 0,
		["oamAddressOffset"] = 4096,
		["screenBrightness"] = 15,
		["windowMaskMainBg2"] = false,
		["oamRamAddress"] = 0,
		["extBgEnabled"] = false,
		["colorMathClipMode"] = 0,
		["oamBaseAddress"] = 16384,
		["windowMaskMainBg3"] = false,
		["cgramAddress"] = 0,
		["hiResMode"] = false,
		["ppu2OpenBus"] = 84,
		["vramAddress"] = 24576,
		["mosaicSize"] = 1,
		["mainScreenLayers"] = 21,
		["hClock"] = 1364,
		["mode7"] = 
		{
			["hScroll"] = 374,
			["verticalMirroring"] = false,
			["vScroll"] = 1788,
			["centerX"] = 128,
			["horizontalMirroring"] = true,
			["valueLatch"] = 6,
			["fillWithTile0"] = false,
			["centerY"] = 256,
			["matrix"] = 
			{
				[1] = 0,
				[2] = 0,
				[3] = 0,
				[0] = 0,
			},
			["largeMap"] = false,
		},
		["cgramWriteBuffer"] = 255,
		["bgMode"] = 1,
		["oamMode"] = 0,
		["ppu1OpenBus"] = 0,
		["vramAddressRemapping"] = 0,
		["vramReadBuffer"] = 3072,
		["cycle"] = 340,
		["windowMaskSubBg3"] = false,
		["windowMaskLogicBg0"] = false,
		["forcedVblank"] = true,
		["mode1Bg3Priority"] = true,
		["cgramAddressLatch"] = false,
		["windows"] = 
		{
			[1] = 
			{
				["invertedLayers"] = 0,
				["left"] = 0,
				["right"] = 0,
				["activeLayers"] = 0,
			},
			[0] = 
			{
				["invertedLayers"] = 0,
				["left"] = 0,
				["right"] = 8,
				["activeLayers"] = 0,
			},
		},
		["windowMaskSubSprites"] = false,
		["objInterlace"] = false,
		["windowMaskSubBg2"] = false,
		["windowMaskSubBg1"] = false,
		["vramIncrementValue"] = 1,
		["windowMaskLogicColor"] = false,
		["subScreenLayers"] = 2,
	},
}
```