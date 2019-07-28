# Formats

[Fable](https://en.wikipedia.org/wiki/Fable_(video_game_series) has unique formats. Below are descriptions these formats, with most information derived from [Fable TLC Mod Wiki](http://fabletlcmod.com/wiki/doku.php) and my own experimentation.

## WAD

A WAD file starts with a header:

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

A file list entry looks like:

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

A LEV file starts with a primary header:

| Field              | Length | Description                                          |
|--------------------|--------|------------------------------------------------------|
| `headerSize`       | 4      | Header size. Should be `25`                          |
| `version`          | 4      | Version number (according to the wiki)               |
| `unknown1`         | 5      | Unknown (ould be zero then a number?)                |
| `obsoleteOffset    | 4      | Offset to obsolete data in the file. (Is this true?) |
| `unknown2`         | 4      | Unknown (could be a number?)                         |
| `navigationOffset` | 4      | Offset to navigation data                            |
| `unknown3`         | 5      | Unknown                                              |

And is followed by a map data header:

| Field           | Length | Description               |
|-----------------|--------|---------------------------|
| `mapHeaderSize` | 1      | Map section's header size |
| `mapVersion`    | 4      | Map version               |
| `uniqueIdCount` | 8      | Number of unique IDs      |
| `width`         | 4      |                           |
| `height`        | 4      |                           |
| `alwaysTrue`    | 1      | Should be `1`             |

Work in progress

## BIG

A BIG file starts with a header.

| Field         | Length | Description              |
|---------------|--------|--------------------------|
| `magicNumber` | 4      | Should be `"BIGB"`       |
| `version`     | 4      | File's version number    |
| `bankOffset`  | 4      | Offset where banks start |
| `unknown1`    | 4      | Unknown                  |

Work in progress.

## License

This documentation is licensed as [CC BY-NC-SA 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/legalcode).