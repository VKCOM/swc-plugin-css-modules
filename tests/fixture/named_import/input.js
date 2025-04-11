import { host, foo as bar } from "./Component.module.css";

const headerLevelClassName = {
  1: host,
  2: bar,
};

const a = host;
const b = notClass;
const c = bar;

classNames(host, platform === "ios" && bar);
