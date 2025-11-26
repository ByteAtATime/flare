import React from "react";
import { Push } from "./actions";

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
const GridDropdown = createComponent("Grid.Dropdown");
const GridDropdownSection = createComponent("Grid.Dropdown.Section");
const GridDropdownItem = createComponent("Grid.Dropdown.Item");

Object.assign(GridDropdown, {
  Section: GridDropdownSection,
  Item: GridDropdownItem,
});

Object.assign(Grid, {
  Section: GridSection,
  Item: GridItem,
  Dropdown: GridDropdown,
});

const List = createComponent("List");
const ListSection = createComponent("List.Section");
const ListItem = createComponent("List.Item");
const ListItemAccessory = createComponent("List.Item.Accessory");
const ListItemDetail = createComponent("List.Item.Detail");
const ListItemDetailMetadata = createComponent("List.Item.Detail.Metadata");
const ListItemDetailMetadataLabel = createComponent(
  "List.Item.Detail.Metadata.Label"
);
const ListItemDetailMetadataLink = createComponent(
  "List.Item.Detail.Metadata.Link"
);
const ListItemDetailMetadataTagList = createComponent(
  "List.Item.Detail.Metadata.TagList"
);
const ListItemDetailMetadataTagListItem = createComponent(
  "List.Item.Detail.Metadata.TagList.Item"
);
const ListItemDetailMetadataSeparator = createComponent(
  "List.Item.Detail.Metadata.Separator"
);

Object.assign(ListItemDetailMetadataTagList, {
  Item: ListItemDetailMetadataTagListItem,
});

Object.assign(ListItemDetailMetadata, {
  Label: ListItemDetailMetadataLabel,
  Link: ListItemDetailMetadataLink,
  TagList: ListItemDetailMetadataTagList,
  Separator: ListItemDetailMetadataSeparator,
});

Object.assign(ListItemDetail, {
  Metadata: ListItemDetailMetadata,
});

Object.assign(ListItem, {
  Detail: ListItemDetail,
  Accessory: ListItemAccessory,
});

const ListDropdown = createComponent("List.Dropdown");
const ListDropdownSection = createComponent("List.Dropdown.Section");
const ListDropdownItem = createComponent("List.Dropdown.Item");

Object.assign(ListDropdown, {
  Section: ListDropdownSection,
  Item: ListDropdownItem,
});

Object.assign(List, {
  Section: ListSection,
  Item: ListItem,
  Dropdown: ListDropdown,
});

const ActionPanel = createComponent("ActionPanel");
const ActionPanelSection = createComponent("ActionPanel.Section");

Object.assign(ActionPanel, {
  Section: ActionPanelSection,
});

const Action = createComponent("Action");

Object.assign(Action, {
  Push,
});

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

export { Grid, List, ActionPanel, Action, Detail };
