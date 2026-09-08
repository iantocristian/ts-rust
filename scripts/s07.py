#!/usr/bin/env python3
"""S07 source preparation. No default command writes reviewed manifests."""

import argparse
import json
from pathlib import Path
import sys

import s07_subset
import s07_subset_freeze


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    export = commands.add_parser("observe-subset", help="run the full pinned-Go syntax/configuration observation")
    export.add_argument("--output", type=Path, required=True)
    candidate = commands.add_parser("subset-candidate", help="classify a full source observation into review candidates")
    candidate.add_argument("--observations", type=Path, required=True)
    candidate.add_argument("--output", type=Path, required=True)
    review = commands.add_parser("subset-review", help="join source candidates with actual loader closure and checker obligations")
    review.add_argument("--observations", type=Path, required=True)
    review.add_argument("--loader-observations", type=Path, required=True)
    review.add_argument("--output", type=Path, required=True)
    freeze = commands.add_parser("freeze-subset", help="check the reviewed full subset; never report partial freeze success")
    freeze.add_argument("--write-manifest", action="store_true")
    freeze.add_argument("--observations", type=Path, default=Path("target/s07-subset/source-observations.ndjson"))
    freeze.add_argument("--loader-observations", type=Path, default=Path("target/s07-subset/review/loading-observations.candidate.json"))
    freeze.add_argument("--review", type=Path, default=Path("data/s07/subset-review.json"))
    args = parser.parse_args()
    if args.command == "observe-subset":
        s07_subset.export_observations(args.output)
    elif args.command == "subset-candidate":
        print(json.dumps(s07_subset.prepare(args.observations, args.output), sort_keys=True))
    elif args.command == "subset-review":
        print(json.dumps(s07_subset_freeze.prepare_review(args.observations, args.loader_observations, args.output), sort_keys=True))
    else:
        print(json.dumps(s07_subset_freeze.freeze(args.observations, args.loader_observations, args.review, args.write_manifest), sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, RuntimeError, OSError) as error:
        print(str(error), file=sys.stderr)
        raise SystemExit(1) from error
