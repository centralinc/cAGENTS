import { spawnSync } from "node:child_process";
import { resolveBinary } from "./binary";

export function run(args: string[] = []) {
  const bin = resolveBinary();
  const res = spawnSync(bin, args, { stdio: "inherit" });
  return res.status ?? 1;
}
