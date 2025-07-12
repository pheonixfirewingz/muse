export class PlaylistData {
  constructor(
    public name: string,
    public owner: string,
    public isPublic: boolean = false
  ) {}
}