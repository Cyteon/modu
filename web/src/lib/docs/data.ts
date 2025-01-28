import { Home, Baseline, File, FileBox, Equal, TriangleAlert, Server, Library, AppWindowIcon, Box, Braces } from "lucide-svelte"

export default {
    pages: [
        {
            "path": "quickstart",
            "title": "Quickstart",
            "icon": Home,
        }, 
        {
            "path": "basics",
            "title": "Basics",
            "icon": Baseline,
        },
        {
            "path": "imports",
            "title": "Imports",
            "icon": FileBox,
        },
        {
            "path": "math",
            "title": "Math",
            "icon": Equal,
        },
        {
            "path": "libraries",
            "title": "Libraries",
            "icon": Library,
        },
        {
            "path": "file",
            "title": "File I/O",
            "icon": File,
        },
        {
            "path": "os",
            "title": "OS Lib",
            "icon": AppWindowIcon,
        },
        {
            "path": "ffi",
            "title": "FFI",
            "icon": Box,
        },
        {
            "path": "json",
            "title": "JSON",
            "icon": Braces,
        },
        /*{
            "path": "limitations",
            "title": "Limitations",
            "icon": TriangleAlert,
        },*/
        {
            "path": "server",
            "title": "Server",
            "icon": Server,
        }
    ]
}