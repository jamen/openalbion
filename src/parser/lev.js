import { promisify } from 'util'
import { open, close, stat, fstat, read, write, createReadStream, createWriteStream } from 'fs'
import mkdirp from 'mkdirp'
import path from 'path'

const openAsync = promisify(open)
const fstatAsync = promisify(fstat)
const statAsync = promisify(stat)
const readAsync = promisify(read)
const closeAsync = promisify(close)
const writeAsync = promisify(write)
const mkdirpAsync = promisify(mkdirp)

export async function parseFile (file) {
    const fd = await openAsync(file, 'r')
    const results = await parse(fd)
    await openAsync(file, 'r')
    return results
}

export async function parse (fd) {
    const header = Buffer.alloc(46)
    await readAsync(fd, header, 0, 46, 0)

    const headerSize = header.readUInt32LE(0)
    const version = header.readUInt32LE(4)

    // 5 unknown bytes. could be a zero byte then a uint32

    const obsoleteOffset = header.readUInt32LE(13) // ?

    // 4 unknown bytes. could be a uint32 like before

    const navigationOffset = header.readUInt32LE(21)

    // 5 unknown bytes

    const mapHeaderSize = header.readUInt8(46)
    const mapVersion = header.readUInt32BE(47)

    // Electron doesn't support this yet
    // const uniqueIdCount = header.readBigUInt64LE(30)

    const uniqueIdCount = (BigInt(header.readUInt32LE(34)) << 32n) | BigInt(header.readUInt32LE(30))

    const width = header.readUInt32LE(38)
    const height = header.readUInt32LE(42)

    // Not useful
    // const alwaysTrue = header.readUInt8LE(46) // ?

    return {
        headerSize,
        version,
        obsoleteOffset,
        navigationOffset,
        uniqueIdCount,
        width,
        height,
        mapHeaderSize,
        mapVersion
        // alwaysTrue
    }
}