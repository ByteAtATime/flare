import { jsx } from "react/jsx-runtime";
import React from "react";

const createComponent = (name: string) => {
  const ComponentFactory = ({
    key,
    ...props
  }: {
    children?: React.ReactNode;
    key?: string | number;
  }) => {
    return jsx(name as React.ElementType, props, key);
  };
  ComponentFactory.displayName = name;
  return ComponentFactory;
};

const Grid = createComponent("Grid");
const GridSection = createComponent("Grid.Section");
const GridItem = createComponent("Grid.Item");

Object.assign(Grid, {
  Section: GridSection,
  Item: GridItem,
});

const ActionPanel = createComponent("ActionPanel");
const ActionPanelSection = createComponent("ActionPanel.Section");

Object.assign(ActionPanel, {
  Section: ActionPanelSection,
});

const Action = createComponent("Action");

const Detail = createComponent("Detail");

export { Grid, ActionPanel, Action, Detail };
