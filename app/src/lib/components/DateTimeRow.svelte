<script>
  export let datetime;
  import { TableBodyCell, Tooltip } from "flowbite-svelte";
  import { formatRelativeDate, readableDate } from "$lib/date";
</script>

<TableBodyCell
  >{datetime ? formatRelativeDate(datetime) : "Just now"}</TableBodyCell
>
<Tooltip placement="bottom">{readableDate(datetime)}</Tooltip>
