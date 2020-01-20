import { app, BrowserWindow } from 'electron'
import path from 'path'

const devMode = process.env.NODE_ENV === 'development'

app.on('ready', () => {
    const browserWindow = new BrowserWindow({
        width: 1600,
        height: 900,
        minWidth: 1024,
        minHeight: 600,
        autoHideMenuBar: true,
        show: false,
        center: true,
        webPreferences: {
            devTools: devMode,
            nodeIntegration: true,
        }
    })

    browserWindow.on('ready-to-show', () => browserWindow.show())

    browserWindow.loadURL(`file://${path.join(__dirname, 'index.html')}`)
})

app.on('window-all-closed', () => {
    app.quit()
})