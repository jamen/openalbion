import { promisify } from 'util'
import { open, close, read, readFile } from 'fs'
import { readWadFd } from './wad'
import path from 'path'

const openAsync = promisify(open)
const readAsync = promisify(read)
const readFileAsync = promisify(readFile)
const closeAsync = promisify(close)

interface Lev {
    headerSize: number,
    version: number,
    obsoleteOffset: number,
    navigationOffset: number,
    uniqueIdCount: BigInt,
    width: number,
    height: number,
    mapHeaderSize: number,
    mapVersion: number,
    heightmapPalette: Buffer,
    ambientSoundVersion: number,
    soundThemesCount: number,
    soundPalette: Buffer,
    soundThemes: string[],
    checksum: number
    heightmap: LevHeightmapCell[],
    soundmap: LevSoundmapCell[]
}

interface LevHeightmapCell {
    size: number,
    version: number,
    height: number,
    groundTheme: Buffer,
    groundThemeStrength: Buffer,
    walkable: boolean,
    passover: boolean,
    soundTheme: number,
    shore: boolean
}

interface LevSoundmapCell {
    size: number,
    version: number,
    soundTheme: Buffer,
    soundThemeStrength: Buffer,
    soundIndex: number
}

/**
 * Parse Lev from inside a Wad file.
 */

export async function readLevFromWadFile (wadFile: string, levFile: string): Promise<Lev> {
    const fd = await openAsync(wadFile, 'r')

    let result = null

    try {
        result = await readLevFromWadFd(fd, levFile)
    } catch (error) {
        await closeAsync(fd)
        throw error
    }

    await closeAsync(fd)

    return result
}

export async function readLevFromWadFd (wadFd: number, levFile: string): Promise<Lev> {
    const wad = await readWadFd(wadFd)

    levFile = path.normalize(levFile.toLowerCase())

    const levEntry = wad.entries.find(file => path.normalize(file.name.toLowerCase()) === levFile)

    if (!levEntry) {
        throw new Error('Lev file ' + levEntry + ' does not exist')
    }

    const levData = Buffer.alloc(levEntry.length)
    await readAsync(wadFd, levData, 0, levEntry.length, levEntry.offset)

    return readLevBuffer(levData)
}

/**
 * Parse an extracted Lev file.
 */

export async function readLevFile (levFile: string): Promise<Lev> {
    return readLevBuffer(await readFileAsync(levFile))
}

/**
 * Parse Lev buffer.
 */

export function readLevBuffer (levData: Buffer): Lev {
    let offset = -4

    const headerSize = levData.readUInt32LE(offset += 4)
    const version = levData.readUInt16LE(offset += 2)

    // 3 unknwon byte
    offset += 3
    // 4 unknown bytes
    offset += 4

    // Could be useful?
    const obsoleteOffset = levData.readUInt32LE(offset += 4)

    // 4 unknown bytes. could be a uint32 like before
    offset+= 4

    const navigationOffset = levData.readUInt32LE(offset += 4)

    // file header ends and map header starts

    const mapHeaderSize = levData.readUInt8(offset += 1)
    const mapVersion = levData.readUInt32BE(offset += 4)

console.log('headerSize', headerSize)
console.log('version', version)

    // Electron doesn't support this yet
    // const uniqueIdCount = header.readBigUInt64LE(30)
    const uniqueIdCount = (BigInt(levData.readUInt32LE(offset += 4)) << 32n) | BigInt(levData.readUInt32LE(offset += 4))

    const width = levData.readUInt32LE(offset += 4)
    const height = levData.readUInt32LE(offset += 4)

    // This is a thing. Useless
    // const alwaysTrue = header.readUInt8LE(46)
    offset += 1

    // Theme data
    const heightmapPalette = levData.slice(offset, offset += 33792)
    const ambientSoundVersion = levData.readUInt32LE(offset += 4)
    const soundThemesCount = levData.readUInt32LE(offset += 4)
    const soundPalette = levData.slice(offset, offset += 33792)
    const checksum = levData.readUInt32LE(offset += 4)

    console.log('soundThemesCount', soundThemesCount)

    console.log('offset', offset)

    offset += 4

    // Sound themes
    let soundThemesIter = soundThemesCount - 1
    const soundThemes = []

    while (soundThemesIter--) {
        const length = levData.readUInt32LE(offset)
        const name = levData.slice(offset += 4, offset += length)

        console.log(length, name.slice(0, 100).toString())

        soundThemes.push(name.toString())
    }

    console.log('soundThemes', soundThemes)

    // console.log(soundThemes)

    // Heightmap
    let heightmapIter = (width + 1) * (height + 1)
    const heightmap: LevHeightmapCell[] = []

    while (heightmapIter--) {
        const size = levData.readUInt32LE(offset)
        const version = levData.readUInt8(offset + 4)
        const height = levData.readFloatLE(offset + 5)
        // const zero = levData.readUInt32LE(offset + 9)
        const groundTheme = levData.slice(offset + 10, offset + 13)
        const groundThemeStrength = levData.slice(offset + 13, offset + 15)
        const walkable = !!levData.readUInt8(offset + 15)
        const passover = !!levData.readUInt8(offset + 16)
        const soundTheme = levData.readUInt8(offset + 17)
        // const zero = levData.readUint8(offset + 18)
        const shore = !!levData.readUInt8(offset + 19)
        // const unknown = levData.readUint8(offset + 20)

        offset += 21

        heightmap.push({
            size,
            version,
            height,
            groundTheme,
            groundThemeStrength,
            walkable,
            passover,
            soundTheme,
            shore
        })
    }

    // console.log('offset', offset)
    // console.log('heightmap length', heightmap.length)
    // console.log('heightmap start', heightmap.slice(0, 5))
    // console.log('heightmap end', heightmap.slice(-5))

    // Sound Map
    // let soundMapIter = width * height
    let soundMapIter = 3328
    const soundmap: LevSoundmapCell[] = []

    while (soundMapIter--) {
        const size = levData.readUInt32LE(offset += 4)
        const version = levData.readUInt8(offset += 1)
        const soundTheme = levData.slice(offset, offset += 3)
        const soundThemeStrength = levData.slice(offset, offset += 2)
        const soundIndex = levData.readUInt8(offset += 1)

        soundmap.push({
            size,
            version,
            soundTheme,
            soundThemeStrength,
            soundIndex
        })
    }

    // console.log(soundmap.length, soundmap)

    console.log('offsets', offset, navigationOffset)

    return {
        headerSize,
        version,
        obsoleteOffset,
        navigationOffset,
        uniqueIdCount,
        width,
        height,
        mapHeaderSize,
        mapVersion,
        heightmapPalette,
        ambientSoundVersion,
        soundThemesCount,
        soundPalette,
        checksum,
        soundThemes,
        heightmap,
        soundmap,
    }
}