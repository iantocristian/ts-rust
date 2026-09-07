// This adapter is copied beside the pinned encoder generator in the tooling
// checkout. Its carried patch only exposes these existing analysis functions.
import * as fs from "node:fs";
import { api } from "./schema.ts";
import { analyzeNode, childType, getAutoEncodedLayout, unionKindValues } from "./generate-encoder.ts";

const output = process.argv[2];
if (!output) throw new Error("expected encoder schema output path");
const nodes = api.nodes().map(node => {
    const info = analyzeNode(node);
    return {
        name: node.name,
        kinds: node.allKinds().map(kind => kind.name),
        dataType: info.dataType,
        handWrittenCommonData: info.handWrittenCommonData,
        handWritten: !!node.handWritten,
        textMember: info.textMember ? api.uncapitalize(info.textMember.name) : null,
        children: info.childProps.map(member => ({
            name: api.uncapitalize(member.name),
            childType: childType(member),
            optional: member.optional,
        })),
        commonData: getAutoEncodedLayout(info).map(({ member, bitPos, bitWidth }) => ({
            name: api.uncapitalize(member.name),
            bitPosition: bitPos,
            bitWidth,
            optional: member.optional,
            kinds: unionKindValues(member.declaredType).map(kind => kind.name),
        })),
    };
});
fs.writeFileSync(output, JSON.stringify({ version: 1, nodes }, null, 2) + "\n");
