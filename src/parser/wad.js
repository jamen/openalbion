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
    await closeAsync(fd)
    return results
}

export async function parse (fd) {
    const header = await parseHeader(fd)
    const files = await parseFileList(fd, header)
    return { header, files }
}

export async function parseHeader (fd) {
    const header = Buffer.alloc(32)
    await readAsync(fd, header, 0, 32, 0)

    // Check magic number "BBBB"
    const magicNumber = header.readUInt32LE(0)

    if (magicNumber !== 1111638594) {
        throw new Error('Not a WAD file')
    }

    // The next 12 bytes are unknown. Assumed to be 3 integers. Possibly versions.
    // const unknown1 = header.readUInt32LE(4)
    // const unknown2 = header.readUInt32LE(8)
    // const unknown3 = header.readUInt32LE(12)

    const blockSize = header.readUInt32LE(16)
    const fileCount = header.readUInt32LE(20)

    // fileCount is repeated for an unknown reason.
    // const fileCount2 = header.readUInt32LE(24)

    const fileListOffset = header.readUInt32LE(28)

    return {
        blockSize,
        fileCount,
        fileListOffset
    }
}

async function parseFileList (fd, header) {
    const files = []
    const stats = await fstatAsync(fd)
    const fileList = Buffer.alloc(stats.size - header.fileListOffset)
    await readAsync(fd, fileList, 0, fileList.length, header.fileListOffset)
    let fileEntryOffset = 0

    while (fileEntryOffset + 40 < fileList.length) {
        // The first 16 bytes are unknown, Assumed to be 4 fields
        const unknown1 = fileList.readUInt32LE(fileEntryOffset)
        const unknown2 = fileList.readUInt32LE(fileEntryOffset + 4)
        const unknown3 = fileList.readUInt32LE(fileEntryOffset + 8)
        const unknown4 = fileList.readUInt32LE(fileEntryOffset + 12)

        const id = fileList.readUInt32LE(fileEntryOffset + 16)

        // The next 4 bytes are unknown
        const unknown5 = fileList.readUInt32LE(fileEntryOffset + 20)

        // The next 8 bytes are the size and offset of the file.
        const size = fileList.readUInt32LE(fileEntryOffset + 24)
        const offset = fileList.readUInt32LE(fileEntryOffset + 28)

        // The next 4 bytes are unknown
        const unknown6 = fileList.readUInt32LE(fileEntryOffset + 32)

        // The next 4 bytes are the size of the subsequent file name
        const nameSize = fileList.readUInt32LE(fileEntryOffset + 36)

        // The next bytes are a file name (length is indicated by nameSize)
        const nameOffset = fileEntryOffset + 40
        const name = fileList.slice(nameOffset, nameOffset + nameSize).toString()
        fileEntryOffset += 40 + nameSize

        // The next 16 bytes are unknown. Omitting them because they never appear to change. I put the value below.
        // const unknown7 = Array.from(fileList.slice(fileEntryOffset, fileEntryOffset + 15))
        // > [ 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 88, 0, 0 ]

        // The next 28 bytes are a timestamp.
        const timestamp1 = new Date(
            fileList.readUInt32LE(fileEntryOffset + 16),
            fileList.readUInt32LE(fileEntryOffset + 20),
            fileList.readUInt32LE(fileEntryOffset + 24),
            fileList.readUInt32LE(fileEntryOffset + 28),
            fileList.readUInt32LE(fileEntryOffset + 32),
            fileList.readUInt32LE(fileEntryOffset + 36),
            fileList.readUInt32LE(fileEntryOffset + 40)
        )

        // The next 28 bytes are another timestamp.
        const timestamp2 = new Date(
            fileList.readUInt32LE(fileEntryOffset + 44),
            fileList.readUInt32LE(fileEntryOffset + 48),
            fileList.readUInt32LE(fileEntryOffset + 52),
            fileList.readUInt32LE(fileEntryOffset + 56),
            fileList.readUInt32LE(fileEntryOffset + 60),
            fileList.readUInt32LE(fileEntryOffset + 64),
            fileList.readUInt32LE(fileEntryOffset + 68)
        )

        // The next 20 bytes are another (less percise) timestamp.
        const timestamp3 = new Date(
            fileList.readUInt32LE(fileEntryOffset + 72),
            fileList.readUInt32LE(fileEntryOffset + 76),
            fileList.readUInt32LE(fileEntryOffset + 80),
            fileList.readUInt32LE(fileEntryOffset + 84),
            fileList.readUInt32LE(fileEntryOffset + 88)
        )

        fileEntryOffset += 92

        files.push({
            id,
            size,
            offset,
            name,
            timestamp1,
            timestamp2,
            timestamp3,
            unknown1,
            unknown2,
            unknown3,
            unknown4,
            unknown5,
            unknown6,
            // unknown7,
        })
    }

    return files
}

/**
 * Extract WAD files.
 */
export async function extractWad (file, output, options = {}) {
    const fd = await openAsync(file, 'r')

    const { header, files } = await parse(fd)
    const { include, exclude } = options

    const dirsMade = {}
    const written = []

    for (const file of files) {
        if (
            (include && include.indexOf(file.name) === -1) ||
            (exclude && exclude.indexOf(file.name) !== -1)
        ) {
            continue
        }

        const dir = path.resolve(output, path.win32.dirname(file.name))

        if (!dirsMade[dir]) {
            await mkdirpAsync(dir)
            dirsMade[dir] = true
        }

        const archiveFile = createReadStream(null, {
            fd,
            start: file.offset,
            end: file.offset + file.size - 1,
            autoClose: false
        })

        const outFile = createWriteStream(path.resolve(output, file.name))

        await new Promise((resolve, reject) => {
            archiveFile.pipe(outFile)
            .on('close', () => resolve())
            .on('error', err => reject(err))
        })

        written.push(file.name)
    }

    await closeAsync(fd)

    return { header, files, written }
}

/**
 * Patch source files into a WAD archive. Returns the parsed result
 */
export async function patchWad (input, output, patches) {
    const inputFd = await openAsync(input, 'r')
    const inputHeader = await parseHeader(inputFd)
    const inputFiles = await parseFiles(inputFd, inputHeader)
    const blockSize = inputHeader.blockSize

    // Replace input files with patch files
    const files = []
    let offset = blockSize

    for (let inputFile of inputFiles) {
        const name = inputFile.name

        // Skip files we've already seen.
        if (!seen[name]) {
            const patchFile = patches[inputFile.name]

            const file = {
                unknown1: inputFile.unknown1,
                unknown2: inputFile.unknown2,
                unknown3: inputFile.unknown3,
                unknown4: inputFile.unknown4,
                id: files.length + 1,
                unknown5: inputFile.unknown5,
                size: inputFile.size,
                offset,
                unknown6: inputFile.unknown6,
                name,
                timestamp1: inputFile.timestamp1,
                timestamp2: inputFile.timestamp2,
                timestamp3: inputFile.timestamp3,
                stream: null
            }

            // Patch input file
            if (patchFile) {
                const stat = await statAsync(patchFile.path)
                file.size = stat.size
                file.stream = createReadStream(patchFile.path)
            } else {
                file.stream = createReadStream(null, {
                    fd: inputFd,
                    start: inputFile.offset,
                    end: inputFile.offset + inputFile.size - 1,
                    autoClose: false
                })
            }

            files.push(file)

            seen[name] = true

            // The next file's offset is the current file's size padded to the next block.
            offset += Math.ceil(file.size / blockSize) * blockSize
        }
    }

    // Copy old header
    const outputFd = await openAsync(output, 'w')
    const headerData = Buffer.alloc(blockSize)

    await readAsync(inputFd, headerData, 0, blockSize, 0)

    // Write the header.
    headerData.writeUInt32LE(files.length, 20)
    headerData.writeUInt32LE(fileListOffset, 28)

    await writeAsync(outputFd, headerData, 0, blockSize, 0)

    // Done with the input.
    await closeAsync(inputFd)

    // Write files
    const writeArchiveStream = createWriteStream(null, {
        fd: outputFd,
        start: file.offset,
        end: file.offset + file.start - 1,
        autoClose: false
    })

    const fileList = []
    const fileListOffset = offset

    for (const file of files) {
        // Write file contents
        file.stream.pipe(writeArchiveStream)

        await new Promise((resolve, reject) => {
            file.stream.on('end', () => resolve())
            file.stream.on('error', error => reject(error))
        })

        // Create file entry prefix
        const entryPrefix = Buffer.alloc(40)

        entryPrefix.writeUInt32LE(file.unknown1)
        entryPrefix.writeUInt32LE(file.unknown2, 4)
        entryPrefix.writeUInt32LE(file.unknown3, 8)
        entryPrefix.writeUInt32LE(file.unknown4, 12)
        entryPrefix.writeUInt32LE(file.id, 16)
        entryPrefix.writeUInt32LE(file.unknown5, 20)
        entryPrefix.writeUInt32LE(file.size, 24)
        entryPrefix.writeUInt32LE(file.offset, 28)
        entryPrefix.writeUInt32LE(file.unknown6, 32)
        entryPrefix.writeUInt32LE(file.nameSize, 36)

        // After the prefix we put the name, and then the suffix
        const fileName = Buffer.from(file.name)

        const entrySuffix = Buffer.alloc(91)

        // Write timestamps. Maybe useless.
        entrySuffix.writeUInt32LE(file.timestamp1.getFullYear())
        entrySuffix.writeUInt32LE(file.timestamp1.getMonth(), 4)
        entrySuffix.writeUInt32LE(file.timestamp1.getDay(), 8)
        entrySuffix.writeUInt32LE(file.timestamp1.getHours(), 12)
        entrySuffix.writeUInt32LE(file.timestamp1.getSeconds(), 16)
        entrySuffix.writeUInt32LE(file.timestamp1.getMilliseconds(), 20)

        entrySuffix.writeUInt32LE(file.timestamp2.getFullYear(), 24)
        entrySuffix.writeUInt32LE(file.timestamp2.getMonth(), 28)
        entrySuffix.writeUInt32LE(file.timestamp2.getDay(), 32)
        entrySuffix.writeUInt32LE(file.timestamp2.getHours(), 36)
        entrySuffix.writeUInt32LE(file.timestamp2.getSeconds(), 40)
        entrySuffix.writeUInt32LE(file.timestamp2.getMilliseconds(), 44)

        entrySuffix.writeUInt32LE(file.timestamp3.getFullYear(), 48)
        entrySuffix.writeUInt32LE(file.timestamp3.getMonth(), 62)
        entrySuffix.writeUInt32LE(file.timestamp3.getDay(), 66)
        entrySuffix.writeUInt32LE(file.timestamp3.getHours(), 70)

        // Write unknown values. Maybe useless
        entrySuffix.writeUInt8(1, 79)
        entrySuffix.writeUInt8(88, 87)

        // Combine the buffer parts in the file list.
        fileList.push(entryPrefix, fileName, entrySuffix)

        // Write file entry
        await writeAsync(outputFd, 0, entry, entry.length, offset)

        offset += entry.length
    }

    // Write file list data.
    const fileListData = Buffer.concat(fileList)
    await writeAsync(outputFd, fileListData, 0, fileListData.length, fileListOffset)
    await ftruncateAsync(outputFd, fileListOffset + fileListData.length)

    // Finished.
    await closeAsync(outputFd)

    const header = {
        blockSize,
        fileCount: files.length,
        fileListOffset
    }

    return { header, files }
}
