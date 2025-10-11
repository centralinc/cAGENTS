"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.run = run;
const node_child_process_1 = require("node:child_process");
const binary_1 = require("./binary");
function run(args = []) {
    const bin = (0, binary_1.resolveBinary)();
    const res = (0, node_child_process_1.spawnSync)(bin, args, { stdio: "inherit" });
    return res.status ?? 1;
}
