const { app, BrowserWindow } = require('electron')

app.on('ready', () => {
    let browserWindow = new BrowserWindow({
        width: 800,
        height: 600,
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
        browserWindow.openDevTools()
    }
})

app.on('window-all-closed', () => {
    app.quit()
})