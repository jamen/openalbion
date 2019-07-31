import { promisify } from 'util'
import { open, close, stat, fstat, read, write, readFile } from 'fs'
import mkdirp from 'mkdirp'
import { parseWadFd } from './wad'

const openAsync = promisify(open)
const fstatAsync = promisify(fstat)
const statAsync = promisify(stat)
const readAsync = promisify(read)
const readFileAsync = promisify(readFile)
const closeAsync = promisify(close)
const writeAsync = promisify(write)
const mkdirpAsync = promisify(mkdirp)

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

export async function parseLevFromWadFile (wadFile: string, levFile: string): Promise<Lev> {
    const fd = await openAsync(wadFile, 'r')

    let result = null

    try {
        result = await parseLevFromWadFd(fd, levFile)
    } catch (error) {
        await closeAsync(fd)
        throw error
    }

    await closeAsync(fd)

    return result
}

export async function parseLevFromWadFd (wadFd: number, levFile: string): Promise<Lev> {
    const { header, entries } = await parseWadFd(wadFd)

    const levEntry = entries.find(file => file.name === levFile)

    if (!levEntry) {
        throw new Error('Lev file does not exist')
    }

    const levData = Buffer.alloc(levEntry.length)
    await readAsync(wadFd, levData, 0, levEntry.length, levEntry.offset)

    return parseLevBuffer(levData)
}

/**
 * Parse an extracted Lev file.
 */

export async function parseLevFile (levFile: string) {
    return parseLevBuffer(await readFileAsync(levFile))
}

/**
 * Parse Lev buffer.
 */

export async function parseLevBuffer (levData: Buffer) : Promise<Lev> {
    const headerSize = levData.readUInt32LE(0)
    const version = levData.readUInt32LE(4)

    // 5 unknown bytes. could be a zero byte then a uint32

    // Could be useful?
    const obsoleteOffset = levData.readUInt32LE(13)

    // 4 unknown bytes. could be a uint32 like before

    const navigationOffset = levData.readUInt32LE(21)

    // 5 unknown bytes

    const mapHeaderSize = levData.readUInt8(46)
    const mapVersion = levData.readUInt32BE(47)

    // Electron doesn't support this yet
    // const uniqueIdCount = header.readBigUInt64LE(30)
    const uniqueIdCount = (BigInt(levData.readUInt32LE(34)) << 32n) | BigInt(levData.readUInt32LE(30))

    const width = levData.readUInt32LE(38)
    const height = levData.readUInt32LE(42)

    // This is a thing. Useless
    // const alwaysTrue = header.readUInt8LE(46)

    // Theme data
    const heightmapPalette = levData.slice(47, 33839)
    const ambientSoundVersion = levData.readUInt32LE(33839)
    const soundThemesCount = levData.readUInt32LE(33843)
    const soundPalette = levData.slice(33847, 67639)

    // Next there is 4 bytes I don't know what they are, or I messed up the offsets prior.
    // const unknown = levData.readUint32LE(67643)

    let offset = 67643

    // Sound themes
    let soundThemesIter = soundThemesCount - 1
    const soundThemes = []

    while (soundThemesIter--) {
        const length = levData.readUInt32LE(offset)
        const name = levData.slice(offset + 4, offset + 4 + length)
        offset += 4 + length
        soundThemes.push(name.toString())
    }

    // Heightmap
    let heightmapIter = (width + 1) * (height + 1)
    const heightmap: LevHeightmapCell[] = []

    while (heightmapIter--) {
        const size = levData.readUInt32LE(offset)
        const version = levData.readUInt8(offset + 4)
        const height = levData.readUInt32LE(offset + 5)
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

    // Sound Map
    let soundMapIter = width * height
    const soundmap: LevSoundmapCell[] = []

    while (soundMapIter--) {
        const size = levData.readUInt32LE(offset)
        const version = levData.readUInt8(offset + 4)
        const soundTheme = levData.slice(offset + 5, offset + 8)
        const soundThemeStrength = levData.slice(offset + 8, offset + 10)
        const soundIndex = levData.readUInt8(offset + 10)

        offset += 12

        soundmap.push({
            size,
            version,
            soundTheme,
            soundThemeStrength,
            soundIndex
        })
    }

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
        soundThemes,
        heightmap,
        soundmap,
    }
}