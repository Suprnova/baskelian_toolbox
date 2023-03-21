# Baskelian Toolbox
Baskelian Toolbox is a collection of tools used to work with files found in the PlayStation 2 game Baskelian (バスケリアン) released by Jorudan in 2003. It may additionally work with similar files from some of Jorudan's other PS2 games.

# Features
Current features only allow for the extracting of the inner DATs from the main DATA.DAT, as well as the individual files found in those inner DATs.

### Warning
This tool is not designed to work with SOUND.DAT, as that is an entirely different file type.

# Specifications
All data mentioned here is in little endian.

## Primary DAT
| Offset | Type | Variable | Description |
| ------ | ---- | -------- | ----------- |
| 0x00 | u32 | Count | The number of Entries in the table. |
| 0x04 | Entry[] | Entries | An array containing data for every Entry in the table. |
| 0x04 + 0x0C*Count | InnerDAT[] | Data | An array containing the corresponding InnerDAT for each Entry. |

## Entry
| Offset | Type | Variable | Description |
| ------ | ---- | -------- | ----------- |
| 0x00 | u32 | Address | The address of the corresponding InnerDAT within the Primary DAT. |
| 0x04 | u32 | Size | The size of the InnerDAT. |
| 0x08 | u32 | Count | The number of FileEntries in the InnerDAT. |

## InnerDAT
| Offset | Type | Variable | Description |
| ------ | ---- | -------- | ----------- |
| 0x00 | u32 | Count | The number of FileEntries in the table. |
| 0x04 | FileEntry[] | FileEntries | An array containing data for each FileEntry in the table. |
| 0x04 + 0x08*Count | u8[] | Data | An array containing the corresponding file for each FileEntry. |

## FileEntry
| Offset | Type | Variable | Description |
| ------ | ---- | -------- | ----------- |
| 0x00 | u32 | Address | The address of the corresponding file within the InnerDAT. |
| 0x04 | u32 | Size | The size of the file. |

# License

This program is licensed under the Do What The F*ck You Want To Public License (WTFPL). More information can be found at [LICENSE](/LICENSE).