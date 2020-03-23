import { app, BrowserWindow } from 'electron'

app.on('ready', () => {
    const win = new BrowserWindow({
        show: false,
        autoHideMenuBar: true,
        webPreferences: {
            nodeIntegration: true
        }
    })

    win.on('ready-to-show', () => win.show())

    win.loadURL(`file://${__dirname}/index.html`)
})

app.on('window-all-closed', () => app.quit())