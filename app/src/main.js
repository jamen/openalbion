import { app, BrowserWindow } from 'electron'
import { join } from 'path'
import parseArgs from 'mri'

export default function createWindow (options) {
    app.on('ready', () => {
        const window = new BrowserWindow({
            icon: join(__dirname, 'assets/unfable.png'),
            show: false,
            autoHideMenuBar: true,
            webPreferences: {
                nodeIntegration: true
            }
        })

        window.loadURL(`file://${__dirname}/index.html`)

        window.on('ready-to-show', () => {
            window.show()

            if (options.dev) {
                window.openDevTools()
            }
        })
    })

    app.on('window-all-closed', () => {
        app.quit()
    })
}

const cliOptions = {
    boolean: [ '--dev' ]
}

const options = parseArgs(process.argv.slice(2), cliOptions)

createWindow(options)