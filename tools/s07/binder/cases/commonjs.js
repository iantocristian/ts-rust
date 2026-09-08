/** @constructor */ function Box(x) { this.x = x; this.next = undefined; }
Box.prototype.read = function() { return this.x; };
Object.defineProperty(Box.prototype, "size", { get() { return this.x; } });
const old = require("old");
exports.Box = Box;
module.exports.extra = old;
Object.defineProperty(exports, "value", { value: 1 });
Object.defineProperty(module.exports, "default", { get() { return Box; } });
module.exports = { Box, old };
