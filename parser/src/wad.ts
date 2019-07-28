import { promisify } from 'util'
import { open, close, fstat, read, createReadStream, createWriteStream } from 'fs'
import mkdirp from 'mkdirp'
import * as path from 'path'

const openAsync = promisify(open)
const fstatAsync = promisify(fstat)
const readAsync = promisify(read)
const closeAsync = promisify(close)
const mkdirpAsync = promisify(mkdirp)

/**
 * Parse WAD file
 */

interface Wad {
    header: WadHeader,
    entries: WadEntry[]
}

export async function parseWadFile (file: string) : Promise<Wad> {
    const fd = await openAsync(file, 'r')
    const results = await parseWadFd(fd)
    await closeAsync(fd)
    return results
}

export async function parseWadFd (fd: number) : Promise<Wad> {
    // Parse header
    const headerData = Buffer.alloc(32)
    await readAsync(fd, headerData, 0, 32, 0)
    const header = await parseWadHeader(headerData)

    // Parse entries
    const stat = await fstatAsync(fd)
    const entriesData = Buffer.alloc(stat.size - header.entriesOffset)
    await readAsync(fd, entriesData, 0, entriesData.length, header.entriesOffset)
    const entries = await parseWadEntries(entriesData)

    return { header, entries }
}

/**
 * Parse WAD header
 */

interface WadHeader {
    blockSize: number,
    entriesAmount: number,
    entriesOffset: number
}

export async function parseWadHeader (headerData: Buffer) : Promise<WadHeader> {
    // Check magic number "BBBB"
    const magicNumber = headerData.readUInt32LE(0)

    if (magicNumber !== 1111638594) {
        throw new Error('Not a WAD file')
    }

    // The next 12 bytes are unknown. Possibly 3 version numbers.

    const blockSize = headerData.readUInt32LE(16)
    const entriesAmount = headerData.readUInt32LE(20)

    // The next 8 bytes entriesAmount is repeated.

    const entriesOffset = headerData.readUInt32LE(28)

    return {
        blockSize,
        entriesAmount,
        entriesOffset
    }
}

/**
 * Parse WAD file entries
 */

interface WadEntry {
    id: number,
    offset: number,
    length: number,
    name: string,
    timestamp1: Date,
    timestamp2: Date,
    timestamp3: Date,
}

async function parseWadEntries (entriesData: Buffer) : Promise<WadEntry[]> {
    const files = []

    let fileEntryOffset = 0

    while (fileEntryOffset + 40 < entriesData.length) {
        // First 16 bytes are unknown, Assumed to be 4 integer fields

        // Next 8 bytes is the file's index.
        const id = entriesData.readUInt32LE(fileEntryOffset + 16)

        // Next 4 bytes are unknown

        // Next 8 bytes are the length and offset of the file.
        const length = entriesData.readUInt32LE(fileEntryOffset + 24)
        const offset = entriesData.readUInt32LE(fileEntryOffset + 28)

        // Next 4 bytes are unknown

        // Next 4 bytes are the length of the following string
        const nameLength = entriesData.readUInt32LE(fileEntryOffset + 36)

        // Next bytes are a string of the file's name
        const nameOffset = fileEntryOffset + 40
        const name = entriesData.slice(nameOffset, nameOffset + nameLength).toString()
        fileEntryOffset += 40 + nameLength

        // Next 16 bytes are unknown. It appears to never change from
        // [ 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 88, 0, 0 ]

        // The next 28 bytes are a timestamp.
        const timestamp1 = new Date(
            entriesData.readUInt32LE(fileEntryOffset + 16),
            entriesData.readUInt32LE(fileEntryOffset + 20),
            entriesData.readUInt32LE(fileEntryOffset + 24),
            entriesData.readUInt32LE(fileEntryOffset + 28),
            entriesData.readUInt32LE(fileEntryOffset + 32),
            entriesData.readUInt32LE(fileEntryOffset + 36),
            entriesData.readUInt32LE(fileEntryOffset + 40)
        )

        // The next 28 bytes are another timestamp.
        const timestamp2 = new Date(
            entriesData.readUInt32LE(fileEntryOffset + 44),
            entriesData.readUInt32LE(fileEntryOffset + 48),
            entriesData.readUInt32LE(fileEntryOffset + 52),
            entriesData.readUInt32LE(fileEntryOffset + 56),
            entriesData.readUInt32LE(fileEntryOffset + 60),
            entriesData.readUInt32LE(fileEntryOffset + 64),
            entriesData.readUInt32LE(fileEntryOffset + 68)
        )

        // The next 20 bytes are another (less percise) timestamp.
        const timestamp3 = new Date(
            entriesData.readUInt32LE(fileEntryOffset + 72),
            entriesData.readUInt32LE(fileEntryOffset + 76),
            entriesData.readUInt32LE(fileEntryOffset + 80),
            entriesData.readUInt32LE(fileEntryOffset + 84),
            entriesData.readUInt32LE(fileEntryOffset + 88)
        )

        fileEntryOffset += 92

        files.push({
            id,
            offset,
            length,
            name,
            timestamp1,
            timestamp2,
            timestamp3
        })
    }

    return files
}

/**
 * Extract WAD files.
 */

interface ExtractWadOptions {
    include?: string[],
    exclude?: string[]
}

interface ExtractedWad {
    header: WadHeader,
    entries: WadEntry[],
    written: string[]
}

export async function extractWad (file: string, output: string, options: ExtractWadOptions) : Promise<ExtractedWad> {
    const fd = await openAsync(file, 'r')
    const { include, exclude } = options

    const { header, entries } = await parseWadFd(fd)
    const directories: { [path: string]: boolean } = {}
    const written = []

    for (const entry of entries) {
        if (
            (include && include.indexOf(entry.name) === -1) ||
            (exclude && exclude.indexOf(entry.name) !== -1)
        ) {
            continue
        }

        const dir = path.resolve(output, path.win32.dirname(entry.name))

        if (!directories[dir]) {
            await mkdirpAsync(dir)
            directories[dir] = true
        }

        const archiveFile = createReadStream('', {
            fd,
            start: entry.offset,
            end: entry.offset + entry.length - 1,
            autoClose: false
        })

        const outFile = createWriteStream(path.resolve(output, entry.name))

        await new Promise((resolve, reject) => {
            archiveFile.pipe(outFile)
            .on('close', () => resolve())
            .on('error', err => reject(err))
        })

        written.push(entry.name)
    }

    await closeAsync(fd)

    return { header, entries, written }
}