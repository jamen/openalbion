import { promisify } from 'util'
import { open, close, stat, fstat, read, write, createReadStream, createWriteStream } from 'fs'
import mkdirp from 'mkdirp'
import { join } from 'path'

const openAsync = promisify(open)
const fstatAsync = promisify(fstat)
const statAsync = promisify(stat)
const readAsync = promisify(read)
const closeAsync = promisify(close)
const writeAsync = promisify(write)
const mkdirpAsync = promisify(mkdirp)

export async function openFile (fableDirectory) {
    const file = join(fableDirectory, 'data/graphics/graphics.big')
    return openAsync(file, 'r')
}

export async function parse (fd) {
    const header = await parseHeader(fd)
    return { header }
}

export async function parseHeader (fd) {
    const header = Buffer.alloc(16)
    await readAsync(fd, header, 0, 16, 0)

    // Check magic number "BIGB"
    const magicNumber = header.readUInt32LE(0)

    if (magicNumber !== 1111968066) {
        throw new Error('Not a BIG file')
    }

    // Read header fields
    const version = header.readUInt32LE(4)
    const bankAddress = header.readUInt32LE(8)
    const unknown1 = header.readUInt32LE(12)

    return {
        version,
        bankAddress,
        unknown1
    }
}