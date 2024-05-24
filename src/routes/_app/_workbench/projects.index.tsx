import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { useUser } from "@clerk/clerk-react";
import { createFileRoute, Link } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export const Route = createFileRoute('/_app/_workbench/projects/')({
    component: ProjectsIndex,

    loader: async () => {
        const url = await invoke("get_server_url");
        //const projects = await fetch(url + "/projects");
        //console.log(projects)
        return {
            url: url
        }
    }
})

function ProjectsIndex() {
    const [projects, setProjects] = useState([])
    const [managedTeams, setManagedTeams] = useState([])
    const { user } = useUser();
    const owo = Route.useLoaderData();

    if (!user) {
        return null;
    }

    // FIXME dont use useEffect for fetching, lmao
    useEffect(() => {
        fetch(owo.url + "/projects?user=" + user.id)
            .then((res: Response) => res.json())
            .then((boop: any) => { // TODO type the response
                setProjects(boop.projects)
                console.log(boop.projects)
                setManagedTeams(boop.managed_teams)
            })
    }, [])


    return (
        <div className="flex flex-row">
            <div className="flex flex-col mr-16 space-y-2 items-center">
            <h1 className="text-2xl font-semibold">Projects</h1>
            <Dialog>
                <DialogTrigger asChild>
                    <Button variant={"outline"} disabled={managedTeams.length == 0}>Create Project</Button>
                </DialogTrigger>
                <DialogContent> {/** TODO this can/should be broken into its own component */}
                    <DialogHeader>
                        <DialogTitle>Create a new Project</DialogTitle>
                    </DialogHeader>
                    <Input placeholder="Project Name" />
                    <Select>
                        <SelectTrigger>
                            <SelectValue placeholder="Select a team" />
                        </SelectTrigger>
                        <SelectContent>
                            {
                            managedTeams.map((team: any) => {
                                return (
                                    <SelectItem value={team.id} key={team.id}>{team.name}</SelectItem>
                                )
                            })
                            }
                        </SelectContent>
                    </Select>
                    <DialogFooter>
                        <Button>Create</Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
            </div>
            <div className="flex flex-col grow">
                {
                    projects.length == 0 ? 
                    <div>
                        <Skeleton className="mb-2 h-24" />
                        <Skeleton className="mb-2 h-24" />
                        <Skeleton className="mb-2 h-24" />
                    </div> :
                    projects.map((project: any) => {
                        return (
                            <Card className="grid grid-cols-2 items-center mb-2" key={project.id}>
                            <CardHeader className="justify-self-start">
                                <CardTitle>{project.name}</CardTitle>
                                <CardDescription>{project.team}</CardDescription>
                            </CardHeader>
                            <CardContent className="justify-self-end flex flex-row space-x-4 items-center">
                                {/*<p className="text-sm text-muted-foreground">Last updated MM/DD/YYYY, HH:MM:SS</p> */}
                                <Button>
                                    <Link to={"/projects/$pid"} params={{ pid: project.id as string}}>View</Link>
                                </Button>
                            </CardContent>
                            </Card>
                    )
                    })
                }
            </div>
        </div>
    )
}