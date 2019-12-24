const { app, BrowserWindow } = require('electron')

app.on('ready', () => {
    let browserWindow = new BrowserWindow({
        width: 400,
        height: 600,
        resizable: false,
        show: false,
        autoHideMenuBar: true,
        webPreferences: {
            devTools: process.env.NODE_ENV === 'development',
            nodeIntegration: true
        }
    })

    browserWindow.on('closed', () => {
        browserWindow = null
    })

    browserWindow.on('ready-to-show', () => {
        browserWindow.show()
    })

    browserWindow.loadURL(`file://${__dirname}/index.html`)

    if (process.env.NODE_ENV === 'development') {
        browserWindow.openDevTools({ mode: 'detach' })
    }
})

// TODO: Temporary. E.g. closing the browser window when fable launches using less resources, but ensure Fable closes when Defable is truly exited.
app.on('window-all-closed', () => {
    app.quit()
})