const fs = require('fs')
const path = require('path')

const App = require('../out/app.ssr.js')

module.exports = {
    style: fs.readFileSync(path.join(__dirname, '../out/style.css')),
    script: fs.readFileSync(path.join(__dirname, '../out/app.js')),
    html: App.render().html
}