<!-- TODO: Below is AI boilerplate code, think through actual design -->

<script>
  import { 
    Card, 
    Button, 
    Heading, 
    Breadcrumb, 
    BreadcrumbItem,
    Tabs,
    TabItem,
    Avatar,
    Badge,
    Progressbar,
    Dropdown,
    DropdownItem
  } from 'flowbite-svelte';
  
  // Sample data for the dashboard
  const stats = [
    { title: 'Total Users', value: '12,345', change: '+12%' },
    { title: 'Revenue', value: '$34,567', change: '+8%' },
    { title: 'Active Projects', value: '42', change: '+5%' },
    { title: 'Completion Rate', value: '87%', change: '-2%' }
  ];
  
  const recentActivities = [
    { user: 'John Doe', action: 'Created a new project', time: '2 hours ago' },
    { user: 'Jane Smith', action: 'Updated task status', time: '4 hours ago' },
    { user: 'Mike Johnson', action: 'Completed milestone', time: '1 day ago' },
    { user: 'Sarah Williams', action: 'Added new comment', time: '2 days ago' }
  ];
</script>

<main class="container mx-auto p-4">

  <!-- Dashboard Header -->
  <div class="mb-6">
    <Breadcrumb class="mb-4">
      <BreadcrumbItem href="/" home>Home</BreadcrumbItem>
      <BreadcrumbItem>Dashboard</BreadcrumbItem>
    </Breadcrumb>
    
    <div class="flex justify-between items-center">
      <Heading tag="h1" class="text-2xl font-bold">Dashboard Overview</Heading>
      <Button>Export Data</Button>
    </div>
  </div>
  
  <!-- Stats Cards -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
    {#each stats as stat}
      <Card padding="sm">
        <div class="flex flex-col">
          <span class="text-gray-500 text-sm">{stat.title}</span>
          <span class="text-2xl font-bold">{stat.value}</span>
          <span class={stat.change.includes('+') ? 'text-green-500' : 'text-red-500'}>
            {stat.change} from last month
          </span>
        </div>
      </Card>
    {/each}
  </div>
  
  <!-- Main Content -->
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
    <!-- Chart Section -->
    <Card class="lg:col-span-2">
      <Heading tag="h2" class="mb-4">Performance Analytics</Heading>
      <Tabs style="pill">
        <TabItem open title="Weekly">
          <div class="h-64 flex items-center justify-center bg-gray-100 dark:bg-gray-800 rounded">
            <p class="text-gray-500">Chart visualization would go here</p>
          </div>
        </TabItem>
        <TabItem title="Monthly">
          <div class="h-64 flex items-center justify-center bg-gray-100 dark:bg-gray-800 rounded">
            <p class="text-gray-500">Monthly chart data would go here</p>
          </div>
        </TabItem>
        <TabItem title="Yearly">
          <div class="h-64 flex items-center justify-center bg-gray-100 dark:bg-gray-800 rounded">
            <p class="text-gray-500">Yearly chart data would go here</p>
          </div>
        </TabItem>
      </Tabs>
    </Card>
    
    <!-- Activity Feed -->
    <Card>
      <div class="flex justify-between items-center mb-4">
        <Heading tag="h2">Recent Activity</Heading>
        <Button size="xs">View All</Button>
      </div>
      
      <div class="space-y-4">
        {#each recentActivities as activity}
          <div class="flex items-start gap-3 pb-3 border-b border-gray-200 dark:border-gray-700">
            <Avatar size="md" />
            <div class="flex-1">
              <p class="font-medium">{activity.user}</p>
              <p class="text-sm text-gray-500">{activity.action}</p>
              <p class="text-xs text-gray-400">{activity.time}</p>
            </div>
          </div>
        {/each}
      </div>
    </Card>
  </div>
  
  <!-- Project Status Section -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mt-6">
    <!-- Tasks Progress -->
    <Card>
      <Heading tag="h2" class="mb-4">Project Progress</Heading>
      
      <div class="space-y-4">
        <div>
          <div class="flex justify-between mb-1">
            <span class="font-medium">Website Redesign</span>
            <span>75%</span>
          </div>
          <Progressbar progress={75} size="h-2.5" color="blue" />
        </div>
        
        <div>
          <div class="flex justify-between mb-1">
            <span class="font-medium">Mobile App Development</span>
            <span>45%</span>
          </div>
          <Progressbar progress={45} size="h-2.5" color="blue" />
        </div>
        
        <div>
          <div class="flex justify-between mb-1">
            <span class="font-medium">Database Migration</span>
            <span>90%</span>
          </div>
          <Progressbar progress={90} size="h-2.5" color="blue" />
        </div>
      </div>
    </Card>
    
    <!-- Team Members -->
    <Card>
      <Heading tag="h2" class="mb-4">Team Members</Heading>
      
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <Avatar size="md" />
            <div>
              <p class="font-medium">John Doe</p>
              <p class="text-sm text-gray-500">Project Manager</p>
            </div>
          </div>
          <Badge color="blue">Online</Badge>
        </div>
        
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <Avatar size="md" />
            <div>
              <p class="font-medium">Jane Smith</p>
              <p class="text-sm text-gray-500">UI Designer</p>
            </div>
          </div>
          <Badge color="primary">Offline</Badge>
        </div>
        
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <Avatar size="md" />
            <div>
              <p class="font-medium">Mike Johnson</p>
              <p class="text-sm text-gray-500">Developer</p>
            </div>
          </div>
          <Badge color="blue">Online</Badge>
        </div>
      </div>
    </Card>
  </div>

</main>

<style>

</style>
