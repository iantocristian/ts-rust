/** @typedef {{ x: number }} Record */
/** @import { External } from "external" */
/** @template T @param {T} value @returns {T} */
function identity(value) { return value; }
/** @enum {number} */ const Enum = { A: 1, B: 2 };
/** @type {Record} */ const record = { x: 1 };
/** @constructor @extends {Object} */ function Klass() { /** @type {string} */ this.name = ""; }
exports.identity = identity;
