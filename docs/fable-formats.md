# Formats

[Fable](https://en.wikipedia.org/wiki/Fable_(video_game_series) has unique formats. Below are descriptions these formats, with most information derived from [Fable TLC Mod Wiki](http://fabletlcmod.com/wiki/doku.php) and my own experimentation.

## WAD

A WAD file starts with a header

| Field            | Length | Description                                                    |
|------------------|--------|----------------------------------------------------------------|
| `magicNumber`    | 4      | Should be `"BBBB"`                                             |
| `unknown1`       | 12     | Unknown                                                        |
| `blockSize`      | 4      | Amount of bytes the files are segmented into. Should be `2048` |
| `fileCount`      | 4      | Number of files conatined in the archive                       |
| `fileCountAgain` | 4      | Number of files repeated (Why?)                                |
| `fileListOffset` | 4      | Offset where the file list begins                              |
| `padding`        | 2016   | The header is padded with zeros into the size of a block       |

After the header, the file contents are conatined in blocks. A file list follows these blocks (at `fileListOffset`) and is used to parse the files out.

The file list entries are

| Field        | Length     | Description                                                    |
|--------------|------------|----------------------------------------------------------------|
| `unknown2`   | 16         | Unknown                                                        |
| `id`         | 4          | The index of the file in the list                              |
| `unknown3`   | 4          | Unknown                                                        |
| `size`       | 4          | Size of the file                                               |
| `offset`     | 4          | Offset of the file                                             |
| `unknown4`   | 4          | Unknown                                                        |
| `nameSize`   | 4          | Size of the file's name                                        |
| `name`       | `nameSize` | File's name                                                    |
| `unknown4`   | 16         | Unknown (unchanging between files)                             |
| `timestamp1` | 28         | Timestamp with unknown purpose (ctime, atime, or mtime?)       |
| `timestamp2` | 28         | Timestamp with unknown purpose                                 |
| `timestamp3` | 20         | Timestamp with unknown purpose                                 |

Using the `offset` and `size` fields of each file list entry, the blocks preceding the file list can be parsed into their separate files.

The files are supposed to be written to Fable's install directory (as indicated by their paths), and the game can even load the unarchived files (probably for development or pre-distribution purposes). See [userst.init](#) for enabling that.

The unknown fields are thought to be archive information, such as access and modification times, file permissions, or other values unimportant to the game (consdering the unarchived files can be loaded instead).

When the WAD is unarchived, you are left with [TNG](#TNG) and [LEV](#LEV) files.

## LEV

A LEV file starts with a primary header

| Field              | Length | Description                                          |
|--------------------|--------|------------------------------------------------------|
| `headerSize`       | 4      | Header size. Should be `25`                          |
| `version`          | 4      | Version number (according to the wiki)               |
| `unknown1`         | 5      | Unknown (ould be zero then a number?)                |
| `obsoleteOffset    | 4      | Offset to obsolete data in the file. (Is this true?) |
| `unknown2`         | 4      | Unknown (could be a number?)                         |
| `navigationOffset` | 4      | Offset to navigation data                            |
| `unknown3`         | 5      | Unknown                                              |

And is followed by a map data header

| Field           | Length | Description               |
|-----------------|--------|---------------------------|
| `mapHeaderSize` | 1      | Map section's header size |
| `mapVersion`    | 4      | Map version               |
| `uniqueIdCount` | 8      | Number of unique IDs      |
| `width`         | 4      |                           |
| `height`        | 4      |                           |
| `alwaysTrue`    | 1      | Should be `1`             |

Then, some preliminary map data is given.

**I don't know what most of this is  yet, but I'll provide upstream descriptions**

| Field              | Length | Description                                                  |
|--------------------|--------|--------------------------------------------------------------|
| `heightmapPalette` | 33792  | A palette of themes for the heightmap                        |
| `soundVersion`     | 4      | Ambient sound version. 3 is global, and 0 is heightmap sound |
| `soundThemesCount` | 4      | Amount of sound themes                                       |
| `soundPalette`     | 33792  | A pallete of sound themes for the heightmap                  |
| `checksum`         | 4      | A checksum that is only used sometimes                       |

In addition to this is a list of sound theme names.

The list is `soundThemesCount - 1` in length, and contains

| Field        | Length       | Description          |
|--------------|--------------|----------------------|
| `nameLength` | 4            | The size of the name |
| `name`       | `nameLength` | A string name        |

The heightmap data follow this. Its a list of cells that is `(width + 1) * (height + 1)` in length, and contains

| Field                 | Length | Description                                                 |
|-----------------------|--------|-------------------------------------------------------------|
| `size`                | 4      | Cell size (should be the same for each cell)                |
| `version`             | 1      | Cell version                                                |
| `height`              | 4      | Cell height                                                 |
| `zero1`               | 1      | A byte equal to `0`                                         |
| `groundTheme`         | 3      | An array of 3 bytes specifying the ground theme             |
| `groundThemeStrength` | 2      | An array of 2 bytes specifying the ground theme strength    |
| `walkable`            | 1      | A boolean specifying if the hero can walk at this point     |
| `passover`            | 1      | A boolean specifying if the camera can pass over this point |
| `soundTheme`          | 1      | Sound theme (no further description given)                  |
| `zero2`               | 1      | A byte equal to `0`                                         |
| `shore`               | 1      | A boolean specifying if this is a shore point               |
| `unknown`             | 1      | Unknown                                                     |

A "sound map" follow this. Its a list of cells `width * height` in length, and contains

| Field                | Length | Description                                             |
|----------------------|--------|---------------------------------------------------------|
| `size`               | 4      | Size of the cell (should be the same for each cell)     |
| `version`            | 1      | Cell version                                            |
| `soundTheme`         | 3      | An array of 3 bytes specifying the sound theme          |
| `soundThemeStrength` | 2      | An array of 2 bytes specifying the sound theme strength |
| `soundIndex`         | 1      | An index specifying a sound palette item                |

## BIG

A BIG file starts with a header

| Field         | Length | Description              |
|---------------|--------|--------------------------|
| `magicNumber` | 4      | Should be `"BIGB"`       |
| `version`     | 4      | File's version number    |
| `bankOffset`  | 4      | Offset where banks start |
| `unknown1`    | 4      | Unknown                  |

Work in progress.

## License

This documentation is licensed as [CC BY-NC-SA 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/legalcode).