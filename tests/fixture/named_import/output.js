import "./Component.module.css";

const headerLevelClassName = {
  1: "named_host",
  2: "named_foo",
};

const a = "named_host";
const b = notClass;
const c = "named_foo";

classNames("named_host", platform === "ios" && "named_foo");
