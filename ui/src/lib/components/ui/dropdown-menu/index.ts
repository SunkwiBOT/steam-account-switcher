import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui";
import Content from "./dropdown-menu-content.svelte";
import Item from "./dropdown-menu-item.svelte";
import RadioItem from "./dropdown-menu-radio-item.svelte";
import Separator from "./dropdown-menu-separator.svelte";
import Trigger from "./dropdown-menu-trigger.svelte";

const Root = DropdownMenuPrimitive.Root;
const RadioGroup = DropdownMenuPrimitive.RadioGroup;

export {
  Root,
  Content,
  Item,
  RadioGroup,
  RadioItem,
  Separator,
  Trigger,
  Root as DropdownMenu,
  Content as DropdownMenuContent,
  Item as DropdownMenuItem,
  RadioGroup as DropdownMenuRadioGroup,
  RadioItem as DropdownMenuRadioItem,
  Separator as DropdownMenuSeparator,
  Trigger as DropdownMenuTrigger,
};
