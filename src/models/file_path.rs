use substring::Substring;
use crate::DbConnection;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::models::settings::Setting;
use crate::service::file_service::{prepare_podcast_episode_title_to_directory};
use crate::service::path_service::PathService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::utils::error::CustomError;

#[derive(Default, Clone, Debug)]
pub struct FilenameBuilder {
    episode: String,
    raw_episode: PodcastEpisode,
    directory: String,
    suffix: String,
    image_suffix: String,
    image_filename: String,
    filename: String,
    raw_filename: bool,
    podcast: Podcast,
    settings: Setting
}


pub struct FilenameBuilderReturn{
    pub filename: String,
    pub image_filename: String,
    pub local_file_url: String,
    pub local_image_url: String,
}

impl FilenameBuilderReturn {
    pub fn new(filename: String, image_filename: String, local_file_url: String, local_image_url: String) -> Self {
        FilenameBuilderReturn {
            filename,
            image_filename,
            local_file_url,
            local_image_url,
        }
    }
}

impl FilenameBuilder {

    pub fn with_podcast_directory(mut self, directory: &str) -> FilenameBuilder {
        self.directory = directory.to_string();
        self
    }

    pub fn with_episode(mut self, podcast_episode: PodcastEpisode,conn: &mut DbConnection) ->
                                                                                           Result<FilenameBuilder, CustomError> {
        self.episode = prepare_podcast_episode_title_to_directory(podcast_episode.clone(), conn)?;
        self.raw_episode = podcast_episode;
        Ok(self)
    }

    pub fn with_filename(mut self, filename: &str) -> FilenameBuilder {
        self.filename = filename.to_string();
        self
    }

    pub fn with_image_filename(mut self, image_filename: &str) -> FilenameBuilder {
        self.image_filename = image_filename.to_string();
        self
    }

    pub fn with_suffix(mut self, suffix: &str) -> FilenameBuilder {
        self.suffix = suffix.to_string();
        self
    }

    pub fn with_settings(mut self, settings: Setting) -> FilenameBuilder {
        self.settings = settings;
        self
    }

    pub fn with_image_suffix(mut self, image_suffix: &str) -> FilenameBuilder {
        self.image_suffix = image_suffix.to_string();
        self
    }

    pub fn with_raw_directory(mut self,conn: &mut DbConnection) -> Result<FilenameBuilder, CustomError> {
        self.directory = PathService::get_image_path(
            &self.podcast.clone().directory_name,
            Some(self.raw_episode.clone()),
            &self.suffix,
            &self.raw_episode.name,
            conn
        )?;
        self.raw_filename = true;
        Ok(self)
    }

    pub fn with_podcast(mut self, podcast: Podcast) -> FilenameBuilder {
        self.podcast = podcast;
        self
    }

    pub fn build(self,conn: &mut DbConnection)->Result<FilenameBuilderReturn,CustomError>{
        let image_last_slash = self.podcast.image_url.rfind('/').unwrap();
        let binding_substring_for_base_url = self.podcast.image_url.clone();
        let base_url = binding_substring_for_base_url.substring(0, image_last_slash);


        match self.raw_filename{
            true=> {
                match self.settings.direct_paths {
                    true => {
                        let episode_to_encode = format!("/{}", self.episode.clone());
                        let encoded_episode_url = PodcastEpisodeService::map_to_local_url(&episode_to_encode);
                        let resulting_link = format!("{base_url}/{episode_url}.{suffix}",
                                                     base_url = base_url,
                                                     episode_url = encoded_episode_url, suffix = self.suffix.clone());
                        self.create_direct_path_dirs(resulting_link)
                    }
                    false => {
                        let resulting_directory = self.clone()
                            .create_podcast_episode_dir(self.directory.clone(), conn)?;

                        let mut file_paths = self.create_path_dirs(resulting_directory, base_url
                            .to_string())?;
                        file_paths.local_file_url = PodcastEpisodeService::map_to_local_url(&file_paths.local_file_url);
                        file_paths.local_image_url = PodcastEpisodeService::map_to_local_url(&file_paths.local_image_url);
                        Ok(file_paths)
                    }
                }
            }
            false =>{
                match self.settings.direct_paths {
                    true=>{
                        let episode_to_encode = format!("/{}", self.episode.clone());
                        let encoded_episode_url = PodcastEpisodeService::map_to_local_url(&episode_to_encode);
                        let resulting_link = format!("{base_url}{episode_url}", base_url = base_url, episode_url = encoded_episode_url);
                        self.create_direct_path_dirs(resulting_link)
                    }
                    false=>{
                        let sub_episode_path = format!("/{}", self.episode.clone());
                        let resulting_directory = self.clone().create_podcast_episode_dir(format!("{}/{}",self
                            .directory.clone(), self.episode.clone()),conn)?;
                        let resulting_link = format!("{base_url}{}", PodcastEpisodeService::map_to_local_url(&sub_episode_path));

                        let mut file_paths = self.create_path_dirs(resulting_directory,
                                                                   resulting_link)?;
                        file_paths.local_file_url = PodcastEpisodeService::map_to_local_url(&file_paths.local_file_url);
                        file_paths.local_image_url = PodcastEpisodeService::map_to_local_url(&file_paths.local_image_url);
                        Ok(file_paths)
                    }
                }
            }
        }

    }

    fn create_path_dirs(self, resulting_directory: String, resulting_link: String) -> Result<FilenameBuilderReturn, CustomError> {
        Ok(FilenameBuilderReturn::new(format!("{}/{}.{}", resulting_directory,
                                              self.filename.clone(),
                                              self.suffix.clone())
                                      , format!("{}/{}.{}", resulting_directory,
                                                self.image_filename.clone(),
                                                self.image_suffix.clone()),
                                      format!("{}/{}.{}", resulting_link,
                                              self.filename.clone(), self.suffix.clone()),
                                      format!("{}/{}.{}", resulting_link
                                              , self.image_filename.clone(), self.image_suffix.clone())))
    }

    fn create_direct_path_dirs (self, resulting_link: String) -> Result<FilenameBuilderReturn, CustomError> {
        // TODO Adapt this code for direct paths
        Ok(FilenameBuilderReturn::new(format!("{}/{}.{}", self.podcast.directory_name,
                                              self.episode.clone(),
                                              self.suffix.clone())
                                      , format!("{}/{}.{}", self.podcast.directory_name,
                                                self.episode.clone(),
                                                self.image_suffix.clone()),
                                      format!("{}.{}",resulting_link.clone(),self.suffix),
                                      format!("{}.{}",resulting_link, self.image_suffix)))
    }


    fn create_podcast_episode_dir(self,dirname:String,conn: &mut DbConnection)->Result<String, CustomError>{
        PathService::check_if_podcast_episode_directory_available
            (&dirname, self.podcast, conn)
    }
}


#[cfg(test)]
mod tests {
    use crate::models::podcast_episode::PodcastEpisode;
    use crate::models::podcasts::Podcast;

    pub fn get_standard_episode () -> PodcastEpisode{
        let standard_episode:PodcastEpisode = PodcastEpisode{
            id: 0,
            episode_id: "test".to_string(),
            name: "test".to_string(),
            url: "test".to_string(),
            podcast_id: 0,
            image_url: "test".to_string(),
            total_time: 0,
            local_url: "".to_string(),
            local_image_url: "".to_string(),
            description: "test".to_string(),
            status: "".to_string(),
            download_time: None,
            guid: "".to_string(),
            deleted: false,
            file_episode_path: None,
            date_of_recording: "".to_string(),
            file_image_path: None,
        };
        standard_episode
    }

    pub fn get_standard_podcast() -> Podcast {
        let standard_podcast:Podcast = Podcast{
            id: 0,
            name: "".to_string(),
            directory_id: "".to_string(),
            rssfeed: "".to_string(),
            image_url: "/test.img".to_string(),
            summary: None,
            language: None,
            explicit: None,
            keywords: None,
            last_build_date: None,
            author: None,
            active: false,
            original_image_url: "".to_string(),
            directory_name: "".to_string(),
        };
        standard_podcast
    }
}