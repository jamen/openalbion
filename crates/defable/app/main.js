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

app.on('window-all-closed', () => {
    app.quit()
})