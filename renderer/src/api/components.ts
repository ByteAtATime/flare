import React from "react";

const createComponent = (name: string) => {
  const Component = ({
    children,
    ...props
  }: {
    children?: React.ReactNode;
    [key: string]: any;
  }) => {
    const slots: React.ReactNode[] = [];
    const passThroughProps: Record<string, any> = {};

    for (const [key, value] of Object.entries(props)) {
      if (React.isValidElement(value)) {
        slots.push(
          React.createElement("flare-slot", { key, name: key }, value)
        );
      } else {
        passThroughProps[key] = value;
      }
    }

    return React.createElement(name, passThroughProps, ...slots, children);
  };
  Component.displayName = name;
  return Component;
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
const DetailMetadata = createComponent("Detail.Metadata");
const DetailMetadataLabel = createComponent("Detail.Metadata.Label");
const DetailMetadataLink = createComponent("Detail.Metadata.Link");
const DetailMetadataTagList = createComponent("Detail.Metadata.TagList");
const DetailMetadataTagListItem = createComponent(
  "Detail.Metadata.TagList.Item"
);
const DetailMetadataSeparator = createComponent("Detail.Metadata.Separator");

Object.assign(DetailMetadataTagList, {
  Item: DetailMetadataTagListItem,
});

Object.assign(DetailMetadata, {
  Label: DetailMetadataLabel,
  Link: DetailMetadataLink,
  TagList: DetailMetadataTagList,
  Separator: DetailMetadataSeparator,
});

Object.assign(Detail, {
  Metadata: DetailMetadata,
});

export { Grid, ActionPanel, Action, Detail };
