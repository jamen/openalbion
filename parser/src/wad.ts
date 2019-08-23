import { promisify } from 'util'
import { open, close, fstat, read, createReadStream, createWriteStream } from 'fs'
import mkdirp from 'mkdirp'
import * as path from 'path'
import { start } from 'repl';

const openAsync = promisify(open)
const fstatAsync = promisify(fstat)
const readAsync = promisify(read)
const closeAsync = promisify(close)
const mkdirpAsync = promisify(mkdirp)

/**
 * Parse WAD file
 */

interface Wad {
    blockSize: number,
    entriesCount: number,
    entriesOffset: number
    entries: WadEntry[]
}

interface WadEntry {
    id: number,
    offset: number,
    length: number,
    name: string,
    timestamp1: Date,
    timestamp2: Date,
    timestamp3: Date,
}

export async function readWadFile (file: string): Promise<Wad> {
    const fd = await openAsync(file, 'r')
    const results = await readWadFd(fd)
    await closeAsync(fd)
    return results
}

export async function readWadFd (fd: number): Promise<Wad> {
    // Parse header
    const headerData = Buffer.alloc(32)
    await readAsync(fd, headerData, 0, 32, 0)

    // Check magic number "BBBB"
    const magicNumber = headerData.readUInt32LE(0)

    if (magicNumber !== 1111638594) {
        throw new Error('Not a WAD file')
    }

    // The next 12 bytes are unknown. Possibly 3 version numbers.

    const blockSize = headerData.readUInt32LE(16)
    const entriesCount = headerData.readUInt32LE(20)

    // The next 8 bytes entriesCount is repeated.

    const entriesOffset = headerData.readUInt32LE(28)

    // Parse entries
    const stat = await fstatAsync(fd)

    const entriesData = Buffer.alloc(stat.size - entriesOffset)
    await readAsync(fd, entriesData, 0, entriesData.length, entriesOffset)

    const entries = []

    let readEntriesOffset = 0

    while (readEntriesOffset + 40 < entriesData.length) {
        // First 16 bytes are unknown, Assumed to be 4 integer fields

        // Next 8 bytes is the file's index.
        const id = entriesData.readUInt32LE(readEntriesOffset + 16)

        // Next 4 bytes are unknown

        // Next 8 bytes are the length and offset of the file.
        const length = entriesData.readUInt32LE(readEntriesOffset + 24)
        const offset = entriesData.readUInt32LE(readEntriesOffset + 28)

        // Next 4 bytes are unknown

        // Next 4 bytes are the length of the following string
        const nameLength = entriesData.readUInt32LE(readEntriesOffset + 36)

        // Next bytes are a string of the file's name
        const nameOffset = readEntriesOffset + 40
        const name = entriesData.slice(nameOffset, nameOffset + nameLength).toString()
        readEntriesOffset += 40 + nameLength

        // Next 16 bytes are unknown. It appears to never change from
        // [ 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 88, 0, 0 ]

        const timestamp1 = readTimestamp(entriesData, readEntriesOffset + 16, 7)
        const timestamp2 = readTimestamp(entriesData, readEntriesOffset + 44, 7)
        const timestamp3 = readTimestamp(entriesData, readEntriesOffset + 72, 5)

        readEntriesOffset += 92

        entries.push({
            id,
            offset,
            length,
            name,
            timestamp1,
            timestamp2,
            timestamp3
        })
    }

    return {
        blockSize,
        entriesCount: entriesCount,
        entriesOffset,
        entries
    }
}

function readTimestamp (data: Buffer, offset: number, length: number): Date {
    const t = []
    const endOffset = offset + (length * 4)

    while (offset < endOffset) {
        t.push(data.readUInt32LE(offset))
        offset += 4
    }

    return new Date(t[0], t[1], t[2], t[3], t[4], t[5], t[6])
}

/**
 * Extract WAD files.
 *
 * TODO: Create less read streams, allocate more memory to multiple file blocks, and extract faster.
 */

interface ExtractWadOptions {
    include?: string[],
    exclude?: string[]
}

interface ExtractedWad extends Wad {
    written: string[]
}

export async function extractWad (file: string, output: string, options: ExtractWadOptions): Promise<ExtractedWad> {
    const fd = await openAsync(file, 'r')
    const { include = [], exclude = [] } = options

    const wad = await readWadFd(fd)
    const directories: { [path: string]: boolean } = {}
    const written = []

    for (const entry of wad.entries) {
        if (include.indexOf(entry.name) !== -1 || exclude.indexOf(entry.name) === -1) {
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
    }

    await closeAsync(fd)

    return { ...wad, written }
}

